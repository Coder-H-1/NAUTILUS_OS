use crate::driver::hdmi::Hdmi;
use crate::fs::vfs::BlockDevice;
use crate::fs::fat32::Fat32;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

#[repr(packed)]
struct Cbw {
    signature: u32,
    tag: u32,
    data_transfer_length: u32,
    flags: u8,
    lun: u8,
    cb_length: u8,
    cb: [u8; 16],
}

#[repr(align(4))]
struct AlignedCbw(Cbw);

#[repr(packed)]
struct Csw {
    signature: u32,
    tag: u32,
    data_residue: u32,
    status: u8,
}

#[repr(align(4))]
struct AlignedCsw(Csw);

pub struct MassStorage {
    dev_addr: u8,
    bulk_in_ep: u8,
    bulk_out_ep: u8,
    tag: AtomicU32,
    bulk_in_toggle: AtomicBool,
    bulk_out_toggle: AtomicBool,
}

impl MassStorage {
    pub fn new(dev_addr: u8, bulk_in_ep: u8, bulk_out_ep: u8) -> Self {
        MassStorage { 
            dev_addr, 
            bulk_in_ep, 
            bulk_out_ep,
            tag: AtomicU32::new(1),
            bulk_in_toggle: AtomicBool::new(false),  // Start with DATA0
            bulk_out_toggle: AtomicBool::new(false), // Start with DATA0
        }
    }

    fn update_toggle(&self, toggle: &AtomicBool, len: u32) {
        let pktcnt = (len + 63) / 64; // MPS is 64
        if (pktcnt % 2) != 0 {
            toggle.fetch_xor(true, Ordering::SeqCst);
        }
    }

    fn execute_bot(&self, cb: &[u8], data_buf: Option<*mut u8>, data_len: u32, is_in: bool) -> bool {
        let tag = self.tag.fetch_add(1, Ordering::SeqCst);

        let mut cbw = AlignedCbw(Cbw {
            signature: 0x43425355, // 'USBC'
            tag,
            data_transfer_length: data_len,
            flags: if is_in && data_len > 0 { 0x80 } else { 0x00 },
            lun: 0,
            cb_length: cb.len() as u8,
            cb: [0; 16],
        });
        cbw.0.cb[..cb.len()].copy_from_slice(cb);

        let mut urb_cbw = Urb::new(
            self.dev_addr, self.bulk_out_ep, EndpointType::Bulk, UrbDirection::Out,
            64, &mut cbw as *mut _ as *mut u8, 31
        );
        urb_cbw.pid = if self.bulk_out_toggle.load(Ordering::SeqCst) { 2 } else { 0 }; // DATA1 or DATA0
        
        if !submit_urb(&urb_cbw) {
            Hdmi::write_str("USB BOT: CBW send failed\n");
            return false;
        }
        self.update_toggle(&self.bulk_out_toggle, 31);

        if data_len > 0 {
            if let Some(buf) = data_buf {
                let dir = if is_in { UrbDirection::In } else { UrbDirection::Out };
                let ep = if is_in { self.bulk_in_ep } else { self.bulk_out_ep };
                let mut urb_data = Urb::new(
                    self.dev_addr, ep, EndpointType::Bulk, dir,
                    64, buf, data_len
                );
                
                let toggle = if is_in { &self.bulk_in_toggle } else { &self.bulk_out_toggle };
                urb_data.pid = if toggle.load(Ordering::SeqCst) { 2 } else { 0 };
                
                if submit_urb(&urb_data) {
                    self.update_toggle(toggle, data_len);
                } else {
                    Hdmi::write_str("USB BOT: Data phase failed\n");
                    // We still proceed to read CSW to clear device state
                }
            }
        }

        let mut csw = AlignedCsw(Csw { signature: 0, tag: 0, data_residue: 0, status: 0 });
        let mut urb_csw = Urb::new(
            self.dev_addr, self.bulk_in_ep, EndpointType::Bulk, UrbDirection::In,
            64, &mut csw as *mut _ as *mut u8, 13
        );
        urb_csw.pid = if self.bulk_in_toggle.load(Ordering::SeqCst) { 2 } else { 0 };
        
        if !submit_urb(&urb_csw) {
            Hdmi::write_str("USB BOT: CSW receive failed\n");
            return false;
        }
        self.update_toggle(&self.bulk_in_toggle, 13);

        if csw.0.signature != 0x53425355 { // 'USBS'
            Hdmi::write_str("USB BOT: CSW invalid signature\n");
            return false;
        }
        if csw.0.tag != tag {
            Hdmi::write_str("USB BOT: CSW tag mismatch\n");
            return false;
        }
        if csw.0.status != 0 {
            Hdmi::write_str("USB BOT: CSW command failed\n");
            return false;
        }

        true
    }

    pub fn read_capacity(&self) -> bool {
        Hdmi::write_str("USB BOT: SCSI READ CAPACITY(10)...\n");
        let cb: [u8; 10] = [0x25, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut data = [0u32; 2]; // 8 bytes, 4-byte aligned
        if self.execute_bot(&cb, Some(data.as_mut_ptr() as *mut u8), 8, true) {
            Hdmi::write_str("USB BOT: Read Capacity success.\n");
            return true;
        }
        false
    }

    pub fn test_unit_ready(&self) -> bool {
        let cb: [u8; 6] = [0x00, 0, 0, 0, 0, 0];
        self.execute_bot(&cb, None, 0, false)
    }
}

impl BlockDevice for MassStorage {
    fn read_block(&self, lba: u32, buffer: &mut [u8; 512]) -> Result<(), ()> {
        let mut cb = [0u8; 10];
        cb[0] = 0x28; // READ(10)
        let lba_be = lba.to_be_bytes();
        cb[2] = lba_be[0];
        cb[3] = lba_be[1];
        cb[4] = lba_be[2];
        cb[5] = lba_be[3];
        cb[7] = 0;
        cb[8] = 1; // 1 block

        if self.execute_bot(&cb, Some(buffer.as_mut_ptr()), 512, true) {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn init() {
    Hdmi::write_str("USB: Initializing Bulk-Only Transport...\n");
    
    // Debug: Print ADDR_TO_PORT and ADDR_TO_SPEED tables
    Hdmi::write_str("USB BOT: ADDR_TO_PORT=[");
    for i in 0..8 {
        let p = unsafe { crate::driver::usb::ADDR_TO_PORT[i] };
        Hdmi::draw_char((b'0' + p) as char);
        if i < 7 { Hdmi::draw_char(','); }
    }
    Hdmi::write_str("] SPEED=[");
    for i in 0..8 {
        let s = unsafe { crate::driver::usb::ADDR_TO_SPEED[i] };
        Hdmi::draw_char((b'0' + s) as char);
        if i < 7 { Hdmi::draw_char(','); }
    }
    Hdmi::write_str("]\n");
    
    // Auto-detect Mass Storage device on assigned ports
    let mut usb_stick: Option<MassStorage> = None;
    
    for addr in 2..=5 {
        let port = unsafe { crate::driver::usb::ADDR_TO_PORT[addr] };
        if port == 0 {
            Hdmi::write_str("USB BOT: Addr ");
            Hdmi::draw_char((b'0' + addr as u8) as char);
            Hdmi::write_str(" has port=0, skip\n");
            continue;
        }
        
        Hdmi::write_str("USB BOT: Scanning addr ");
        Hdmi::draw_char((b'0' + addr as u8) as char);
        Hdmi::write_str(" port=");
        Hdmi::draw_char((b'0' + port) as char);
        Hdmi::write_str("...\n");
        
        // First try a short 9-byte config descriptor request to get total length
        let mut buf = [0u32; 64]; // 256 bytes, 4-byte aligned
        let buf_ptr = buf.as_mut_ptr() as *mut u8;
        
        if !crate::driver::usb::setup::get_config_descriptor(addr as u8, buf_ptr, 9) {
            Hdmi::write_str("USB BOT: Config desc (9) FAILED\n");
            // Try full 256 anyway
            if !crate::driver::usb::setup::get_config_descriptor(addr as u8, buf_ptr, 256) {
                Hdmi::write_str("USB BOT: Config desc (256) also FAILED\n");
                continue;
            }
        }
        
        let total_len_raw = u16::from_le_bytes([unsafe { *buf_ptr.add(2) }, unsafe { *buf_ptr.add(3) }]);
        Hdmi::write_str("USB BOT: Config desc total_len=");
        // Print number
        let mut tl = total_len_raw;
        let mut tbuf = [0u8; 5];
        let mut ti = 4;
        if tl == 0 {
            Hdmi::draw_char('0');
        } else {
            while tl > 0 {
                tbuf[ti] = (tl % 10) as u8;
                tl /= 10;
                if ti == 0 { break; }
                ti -= 1;
            }
            for j in (ti+1)..=4 {
                Hdmi::draw_char((b'0' + tbuf[j]) as char);
            }
        }
        Hdmi::write_str("\n");
        
        // If we only got 9 bytes, request the full descriptor
        let mut total_len = total_len_raw as usize;
        if total_len > 9 {
            // Re-request with full length
            buf = [0u32; 64];
            let buf_ptr2 = buf.as_mut_ptr() as *mut u8;
            if !crate::driver::usb::setup::get_config_descriptor(addr as u8, buf_ptr2, if total_len > 256 { 256 } else { total_len as u16 }) {
                Hdmi::write_str("USB BOT: Full config desc FAILED\n");
                continue;
            }
        }
        if total_len > 256 { total_len = 256; }
        
        let buf_ptr = buf.as_mut_ptr() as *mut u8;
        let mut offset = 0;
        let mut bulk_in_ep = 0;
        let mut bulk_out_ep = 0;
        let mut found_msc = false;
        
        while offset < total_len {
            let desc_len = unsafe { *buf_ptr.add(offset) } as usize;
            if desc_len < 2 || offset + desc_len > total_len {
                break;
            }
            let desc_type = unsafe { *buf_ptr.add(offset + 1) };
            
            if desc_type == 4 { // Interface Descriptor
                let class = unsafe { *buf_ptr.add(offset + 5) };
                let subclass = unsafe { *buf_ptr.add(offset + 6) };
                let proto = unsafe { *buf_ptr.add(offset + 7) };
                Hdmi::write_str("USB BOT: Iface class=");
                // hex print class
                let hi = (class >> 4) & 0xF;
                let lo = class & 0xF;
                Hdmi::draw_char(if hi < 10 { (b'0' + hi) as char } else { (b'A' + hi - 10) as char });
                Hdmi::draw_char(if lo < 10 { (b'0' + lo) as char } else { (b'A' + lo - 10) as char });
                Hdmi::write_str(" sub=");
                let hi = (subclass >> 4) & 0xF;
                let lo = subclass & 0xF;
                Hdmi::draw_char(if hi < 10 { (b'0' + hi) as char } else { (b'A' + hi - 10) as char });
                Hdmi::draw_char(if lo < 10 { (b'0' + lo) as char } else { (b'A' + lo - 10) as char });
                Hdmi::write_str(" proto=");
                let hi = (proto >> 4) & 0xF;
                let lo = proto & 0xF;
                Hdmi::draw_char(if hi < 10 { (b'0' + hi) as char } else { (b'A' + hi - 10) as char });
                Hdmi::draw_char(if lo < 10 { (b'0' + lo) as char } else { (b'A' + lo - 10) as char });
                Hdmi::write_str("\n");
                
                if class == 0x08 { // Mass Storage Class
                    found_msc = true;
                } else {
                    found_msc = false; // Reset if next interface isn't MSC
                }
            } else if desc_type == 5 && found_msc { // Endpoint Descriptor
                let ep_addr = unsafe { *buf_ptr.add(offset + 2) };
                let attr = unsafe { *buf_ptr.add(offset + 3) };
                if (attr & 3) == 2 { // Bulk Endpoint
                    if (ep_addr & 0x80) != 0 {
                        bulk_in_ep = ep_addr & 0x0F;
                        Hdmi::write_str("USB BOT: Bulk IN EP=");
                        Hdmi::draw_char((b'0' + bulk_in_ep) as char);
                        Hdmi::write_str("\n");
                    } else {
                        bulk_out_ep = ep_addr & 0x0F;
                        Hdmi::write_str("USB BOT: Bulk OUT EP=");
                        Hdmi::draw_char((b'0' + bulk_out_ep) as char);
                        Hdmi::write_str("\n");
                    }
                }
            }
            offset += desc_len;
        }
        
        if found_msc && bulk_in_ep != 0 && bulk_out_ep != 0 {
            Hdmi::write_str("USB BOT: Found Mass Storage on address ");
            Hdmi::draw_char((b'0' + addr as u8) as char);
            Hdmi::write_str("!\n");
            usb_stick = Some(MassStorage::new(addr as u8, bulk_in_ep, bulk_out_ep));
            break;
        } else {
            Hdmi::write_str("USB BOT: Addr ");
            Hdmi::draw_char((b'0' + addr as u8) as char);
            Hdmi::write_str(" not MSC (msc=");
            Hdmi::draw_char(if found_msc { '1' } else { '0' });
            Hdmi::write_str(" in=");
            Hdmi::draw_char((b'0' + bulk_in_ep) as char);
            Hdmi::write_str(" out=");
            Hdmi::draw_char((b'0' + bulk_out_ep) as char);
            Hdmi::write_str(")\n");
        }
    }
    
    if let Some(stick) = usb_stick {
        // SCSI Test Unit Ready with retries to wait for pendrive spin-up
        let mut ready = false;
        Hdmi::write_str("USB BOT: Waiting for unit ready...\n");
        for _ in 0..10 {
            if stick.test_unit_ready() {
                ready = true;
                break;
            }
            crate::core::utils::delay(500); // 500ms between attempts
        }
        
        if ready && stick.read_capacity() {
            Hdmi::write_str("USB: Mass Storage ready. Mounting FAT32...\n");
            let fs = Fat32::new(stick);
            use crate::fs::vfs::FileSystem;
            fs.list_dir("/");
        } else {
            Hdmi::write_str("USB: Mass Storage failed to initialize.\n");
        }
    } else {
        Hdmi::write_str("USB: No Mass Storage device detected.\n");
    }
}
