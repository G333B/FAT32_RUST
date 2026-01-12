use crate::Result;
use crate::Fat32Error;

// bootsecteur pour fat32

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct BootSector {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    // FAT32 spécifique
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

impl BootSector {
    // use unsafe pour lire depuis la memoire une structure packed
    // On s'assure que data contient au moins 512 octets avant d'appeler.
    pub unsafe fn from_bytes(data: &[u8]) -> Self {
        unsafe { core::ptr::read_unaligned(data.as_ptr() as *const BootSector) }
    }

    /// Vérifier que le boot sector est valide
    pub fn validate(&self) -> Result<()> {
        // signature
        if self.boot_signature != 0x28 && self.boot_signature != 0x29 {
            return Err(Fat32Error::InvalidBootSector);
        }

        // Vérifier bytes per sector
        if self.bytes_per_sector != 512 
            && self.bytes_per_sector != 1024 
            && self.bytes_per_sector != 2048 
            && self.bytes_per_sector != 4096 {
            return Err(Fat32Error::InvalidBootSector);
        }

        // Au moins une FAT
        if self.num_fats == 0 {
            return Err(Fat32Error::InvalidBootSector);
        }

        Ok(())
    }

    /// Taille d'un cluster en octets
    pub fn cluster_size(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }

    /// Taille de la FAT
    pub fn fat_size(&self) -> u32 {
        if self.fat_size_16 != 0 {
            self.fat_size_16 as u32
        } else {
            self.fat_size_32
        }
    }

    /// Nombre total de secteurs
    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16 != 0 {
            self.total_sectors_16 as u32
        } else {
            self.total_sectors_32
        }
    }

    /// Premier secteur de données
    pub fn first_data_sector(&self) -> u32 {
        self.reserved_sector_count as u32 + (self.num_fats as u32 * self.fat_size())
    }

    /// Premier secteur de la FAT
    pub fn first_fat_sector(&self) -> u32 {
        self.reserved_sector_count as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_sector_validation() {
        let mut data = [0u8; 512];
        data[66] = 0x29;
        data[11..13].copy_from_slice(&512u16.to_le_bytes());
        data[13] = 8;
        data[16] = 2;

        let bs = unsafe { BootSector::from_bytes(&data) };
        assert!(bs.validate().is_ok());
    }
}
