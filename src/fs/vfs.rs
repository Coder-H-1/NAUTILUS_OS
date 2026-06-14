use crate::driver::hdmi::Hdmi;

pub fn init() {
    Hdmi::write_str("VFS (Virtual File System) ready.\n");
}

pub trait FileSystem {
    fn read_file(&self, path: &str) -> Option<&[u8]>;
    fn list_dir(&self, path: &str);
}

pub trait BlockDevice {
    fn read_block(&self, lba: u32, buffer: &mut [u8; 512]) -> Result<(), ()>;
}
