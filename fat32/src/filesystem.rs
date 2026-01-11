// filesystem for reading writing  etc etc

use alloc::vec::Vec;
use crate::{BlockDevice, BootSector, DirectoryEntry, Fat32Error, FatTable, Result};

pub struct Fat32FileSystem<D: BlockDevice> {
    device: D,
    boot_sector: BootSector,
    current_directory: u32, // cluster du répertoire courant
}