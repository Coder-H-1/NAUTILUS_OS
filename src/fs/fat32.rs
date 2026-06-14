use crate::driver::hdmi::Hdmi;
use super::vfs::{FileSystem, BlockDevice};

// Real FAT32 structs
#[repr(packed)]
struct BiosParameterBlock {
    jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entries: u16,
    total_sectors_16: u16,
    media: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    flags: u16,
    fat_version: u16,
    root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    sys_id: [u8; 8],
}

pub struct Fat32<T: BlockDevice> {
    device: T,
    partition_lba: u32,
    fat_start_lba: u32,
    data_start_lba: u32,
    sectors_per_cluster: u32,
    root_cluster: u32,
}

impl<T: BlockDevice> Fat32<T> {
    pub fn new(device: T) -> Self {
        Hdmi::write_str("FAT32: Reading MBR to find partition...\n");
        let mut block = [0u8; 512];
        if device.read_block(0, &mut block).is_err() {
            Hdmi::write_str("FAT32: Failed to read MBR\n");
            // Return dummy on error
            return Self::dummy(device);
        }

        // MBR partition 1 entry starts at offset 0x1BE
        let p1_type = block[0x1BE + 4];
        if p1_type != 0x0B && p1_type != 0x0C {
            Hdmi::write_str("FAT32: Partition 1 is not FAT32\n");
            return Self::dummy(device);
        }

        // Read LBA of partition start
        let partition_lba = u32::from_le_bytes([
            block[0x1BE + 8],
            block[0x1BE + 9],
            block[0x1BE + 10],
            block[0x1BE + 11],
        ]);

        Hdmi::write_str("FAT32: Found partition. Reading Boot Sector...\n");
        if device.read_block(partition_lba, &mut block).is_err() {
            Hdmi::write_str("FAT32: Failed to read Boot Sector\n");
            return Self::dummy(device);
        }

        let bpb = unsafe { &*(block.as_ptr() as *const BiosParameterBlock) };
        
        let reserved_sectors = bpb.reserved_sectors as u32;
        let num_fats = bpb.num_fats as u32;
        let sectors_per_fat = bpb.sectors_per_fat_32;
        let sectors_per_cluster = bpb.sectors_per_cluster as u32;
        let root_cluster = bpb.root_cluster;

        let fat_start_lba = partition_lba + reserved_sectors;
        let data_start_lba = fat_start_lba + (num_fats * sectors_per_fat);

        Hdmi::write_str("FAT32: Mount successful!\n");

        Fat32 {
            device,
            partition_lba,
            fat_start_lba,
            data_start_lba,
            sectors_per_cluster,
            root_cluster,
        }
    }

    fn dummy(device: T) -> Self {
        Fat32 {
            device,
            partition_lba: 0,
            fat_start_lba: 0,
            data_start_lba: 0,
            sectors_per_cluster: 0,
            root_cluster: 0,
        }
    }

    pub fn cluster_to_lba(&self, cluster: u32) -> u32 {
        if cluster < 2 { return 0; }
        self.data_start_lba + ((cluster - 2) * self.sectors_per_cluster)
    }
}

impl<T: BlockDevice> FileSystem for Fat32<T> {
    fn read_file(&self, _path: &str) -> Option<&[u8]> {
        Hdmi::write_str("FAT32: read_file logic...\n");
        None 
    }

    fn list_dir(&self, _path: &str) {
        Hdmi::write_str("FAT32: root dir is at cluster ");
        // hacky print number
        if self.root_cluster > 0 {
            Hdmi::write_str("2 (standard)\n");
        } else {
            Hdmi::write_str("unknown\n");
        }
    }
}

pub fn init() {
    Hdmi::write_str("FAT32 driver init called.\n");
    // Actual init happens when a block device is passed to Fat32::new()
}
