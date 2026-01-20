//! Gestion de la table FAT

use alloc::vec::Vec;
use alloc::vec;  // ← Import de la macro vec!
use crate::{BlockDevice, BootSector, Fat32Error, Result};

/// Gère la lecture de la File Allocation Table
pub struct FatTable<'a, D: BlockDevice> {
    device: &'a mut D,
    boot_sector: &'a BootSector,
    cache: Option<(u32, Vec<u8>)>,
}

impl<'a, D: BlockDevice> FatTable<'a, D> {
    pub fn new(device: &'a mut D, boot_sector: &'a BootSector) -> Self {
        Self {
            device,
            boot_sector,
            cache: None,
        }
    }

    /// Obtenir le cluster suivant dans la chaîne
    pub fn next_cluster(&mut self, cluster: u32) -> Result<u32> {
        // Les clusters commencent à 2
        if cluster < 2 {
            return Err(Fat32Error::InvalidCluster);
        }

        // Calculer l'offset dans la FAT
        let fat_offset = cluster * 4;
        let bytes_per_sec = self.boot_sector.bytes_per_sector();
        let fat_sector = self.boot_sector.first_fat_sector() 
            + (fat_offset / bytes_per_sec as u32);
        let entry_offset = (fat_offset % bytes_per_sec as u32) as usize;

        // Lire le secteur de la FAT
        let sector_data = self.read_fat_sector(fat_sector)?;

        // Lire l'entrée (4 octets)
        let entry = u32::from_le_bytes([
            sector_data[entry_offset],
            sector_data[entry_offset + 1],
            sector_data[entry_offset + 2],
            sector_data[entry_offset + 3],
        ]) & 0x0FFFFFFF; // Seulement 28 bits utilisés

        // Interpréter la valeur
        match entry {
            0x0FFFFFF8..=0x0FFFFFFF => Err(Fat32Error::EndOfChain),
            0x00000000 | 0x00000001 => Err(Fat32Error::InvalidCluster),
            cluster => Ok(cluster),
        }
    }

    /// Lire un secteur de la FAT (avec cache)
    fn read_fat_sector(&mut self, sector: u32) -> Result<&Vec<u8>> {
        // Vérifier le cache
        if let Some((cached_sector, _)) = &self.cache {
            if *cached_sector == sector {
                if let Some((_, ref data)) = self.cache {
                    return Ok(data);
                }
            }
        }

        // Lire depuis le disque
        let bytes_per_sec = self.boot_sector.bytes_per_sector();
        let mut buffer = vec![0u8; bytes_per_sec as usize];
        self.device.read_sector(sector, &mut buffer)?;

        // Mettre en cache
        self.cache = Some((sector, buffer));

        if let Some((_, ref data)) = self.cache {
            Ok(data)
        } else {
            unreachable!()
        }
    }

    /// Obtenir tous les clusters d'une chaîne
    pub fn cluster_chain(&mut self, start_cluster: u32) -> Result<Vec<u32>> {
        let mut chain = Vec::new();
        let mut current = start_cluster;

        loop {
            chain.push(current);

            match self.next_cluster(current) {
                Ok(next) => current = next,
                Err(Fat32Error::EndOfChain) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BootSector;

    // Mock device pour les tests
    struct MockDevice {
        data: Vec<u8>,
    }

    impl BlockDevice for MockDevice {
        fn read_sector(&mut self, sector: u32, buffer: &mut [u8]) -> Result<()> {
            let offset = sector as usize * 512;
            buffer.copy_from_slice(&self.data[offset..offset + buffer.len()]);
            Ok(())
        }

        fn write_sector(&mut self, _: u32, _: &[u8]) -> Result<()> {
            Ok(())
        }

        fn sector_size(&self) -> usize {
            512
        }
    }

    #[test]
    fn test_invalid_cluster() {
        let mut device = MockDevice { data: vec![0; 1024 * 512] };
        device.data[66] = 0x29;
        device.data[11..13].copy_from_slice(&512u16.to_le_bytes());
        device.data[13] = 1;
        device.data[16] = 2;
        
        let bs = unsafe { BootSector::from_bytes(&device.data[0..512]) };
        let mut fat = FatTable::new(&mut device, &bs);

        assert!(fat.next_cluster(0).is_err());
        assert!(fat.next_cluster(1).is_err());
    }
}