use crate::Result;
use crate::Fat32Error;

// bootsecteur pour fat32

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct BootSector {}