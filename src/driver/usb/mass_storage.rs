use crate::driver::hdmi::Hdmi;
use crate::fs::vfs::BlockDevice;
use crate::fs::fat32::Fat32;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;

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

#[repr(packed)]
struct Csw {
    signature: u32,
    tag: u32,
    data_residue: u32,
    status: u8,
}

pub struct MassStorage {
    dev_addr: u8,
    bulk_in_ep: u8,
    bulk_out_ep: u8,
}

impl MassStorage {
    pub fn new(dev_addr: u8, bulk_in_ep: u8, bulk_out_ep: u8) -> Self {
        MassStorage { dev_addr, bulk_in_ep, bulk_out_ep }
    }
}

impl BlockDevice for MassStorage {
    fn read_block(&self, lba: u32, buffer: &mut [u8; 512]) -> Result<(), ()> {
        Hdmi::write_str("USB BOT: Sending SCSI READ(10)...\n");
        
        let mut cbw = Cbw {
            signature: 0x43425355, // 'USBC'
            tag: 1,
            data_transfer_length: 512,
            flags: 0x80, // Data IN
            lun: 0,
            cb_length: 10,
            cb: [0; 16],
        };
        
        // SCSI READ(10) command
        cbw.cb[0] = 0x28; // Operation code
        let lba_be = lba.to_be_bytes();
        cbw.cb[2] = lba_be[0];
        cbw.cb[3] = lba_be[1];
        cbw.cb[4] = lba_be[2];
        cbw.cb[5] = lba_be[3];
        cbw.cb[7] = 0; // Transfer length MSB
        cbw.cb[8] = 1; // Transfer length LSB (1 block = 512 bytes)

        // 1. Send CBW
        let urb_cbw = Urb::new(
            self.dev_addr, self.bulk_out_ep, EndpointType::Bulk, UrbDirection::Out,
            64, &mut cbw as *mut _ as *mut u8, 31
        );
        submit_urb(&urb_cbw);

        // 2. Read Data
        let urb_data = Urb::new(
            self.dev_addr, self.bulk_in_ep, EndpointType::Bulk, UrbDirection::In,
            64, buffer.as_mut_ptr(), 512
        );
        submit_urb(&urb_data);

        // 3. Read CSW
        let mut csw = Csw { signature: 0, tag: 0, data_residue: 0, status: 0 };
        let urb_csw = Urb::new(
            self.dev_addr, self.bulk_in_ep, EndpointType::Bulk, UrbDirection::In,
            64, &mut csw as *mut _ as *mut u8, 13
        );
        submit_urb(&urb_csw);

        // Dummy return ok since we are still building the wait logic
        Ok(())
    }
}

pub fn init() {
    Hdmi::write_str("USB: Initializing Bulk-Only Transport...\n");
    // Hardcode generic USB stick device address and endpoints for testing phase
    let usb_stick = MassStorage::new(2, 1, 2);
    Hdmi::write_str("USB: Mass Storage ready. Mounting FAT32...\n");
    
    // Connect actual USB BOT to FAT32 parser
    let fs = Fat32::new(usb_stick);
    
    use crate::fs::vfs::FileSystem;
    fs.list_dir("/");
}
