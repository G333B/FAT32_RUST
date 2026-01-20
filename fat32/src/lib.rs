#![no_std]
extern crate alloc;

pub mod error;
pub mod boot_sector;
pub mod directory;
pub mod fat_table;
pub mod filesystem;

pub use error::{Fat32Error, Result};
pub use boot_sector::BootSector;
pub use directory::{DirectoryEntry, FileAttributes};
pub use fat_table::FatTable;
pub use filesystem::Fat32FileSystem;

pub trait BlockDevice {
    fn read_sector(&mut self, sector: u32, buffer: &mut [u8]) -> Result<()>;
    fn write_sector(&mut self, sector: u32, buffer: &[u8]) -> Result<()>;
    fn sector_size(&self) -> usize;
}