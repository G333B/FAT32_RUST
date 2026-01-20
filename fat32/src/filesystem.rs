//! Système de fichiers FAT32

use alloc::vec::Vec;
use crate::{BlockDevice, BootSector, DirectoryEntry, Fat32Error, FatTable, Result};

pub struct Fat32FileSystem<D: BlockDevice> {
    device: D,
    boot_sector: BootSector,
    current_directory: u32, // cluster du répertoire courant
}

impl<D: BlockDevice> Fat32FileSystem<D> {
    /// Créer un nouveau système de fichiers
    pub fn new(mut device: D) -> Result<Self> {
        // Lire le boot sector
        let mut buffer = alloc::vec![0u8; 512];
        device.read_sector(0, &mut buffer)?;

        let boot_sector = unsafe { BootSector::from_bytes(&buffer) };
        boot_sector.validate()?;

        let current_directory = boot_sector.root_cluster;

        Ok(Self {
            device,
            boot_sector,
            current_directory,
        })
    }

    /// Obtenir le cluster du répertoire courant
    pub fn current_dir(&self) -> u32 {
        self.current_directory
    }

    /// Changer de répertoire
    pub fn change_dir(&mut self, path: &str) -> Result<()> {
        let cluster = self.resolve_path(path)?;
        
        // Vérifier que c'est bien un dossier
        let _ = self.read_directory(cluster)?;
        
        self.current_directory = cluster;
        Ok(())
    }

    /// Lister les fichiers d'un répertoire
    pub fn list_dir(&mut self, path: Option<&str>) -> Result<Vec<DirectoryEntry>> {
        let cluster = if let Some(p) = path {
            self.resolve_path(p)?
        } else {
            self.current_directory
        };

        self.read_directory(cluster)
    }

    /// Lire le contenu d'un fichier
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        // Séparer le chemin et le nom du fichier
        let (dir_cluster, filename) = self.parse_path(path)?;
        let entries = self.read_directory(dir_cluster)?;

        // Trouver le fichier
        let entry = entries
            .iter()
            .find(|e| {
                !e.attributes().is_directory() 
                    && e.short_name().eq_ignore_ascii_case(filename)
            })
            .ok_or(Fat32Error::NotFound)?;

        // Fichier vide
        if entry.file_size == 0 {
            return Ok(Vec::new());
        }

        // Lire tous les clusters
        let mut fat = FatTable::new(&mut self.device, &self.boot_sector);
        let clusters = fat.cluster_chain(entry.first_cluster())?;

        let mut data = Vec::new();
        for cluster in clusters {
            let cluster_data = self.read_cluster(cluster)?;
            data.extend_from_slice(&cluster_data);
        }

        // Tronquer à la vraie taille
        data.truncate(entry.file_size as usize);
        Ok(data)
    }

    /// Résoudre un chemin vers un numéro de cluster
    fn resolve_path(&mut self, path: &str) -> Result<u32> {
        // Chemin absolu ou relatif ?
        let (mut current, remaining) = if path.starts_with('/') {
            (self.boot_sector.root_cluster, &path[1..])
        } else {
            (self.current_directory, path)
        };

        if remaining.is_empty() {
            return Ok(current);
        }

        // Parcourir chaque composant du chemin
        for component in remaining.split('/') {
            if component.is_empty() {
                continue;
            }

            if component == "." {
                continue;
            }

            if component == ".." {
                current = self.find_parent(current)?;
                continue;
            }

            // Chercher dans le répertoire courant
            let entries = self.read_directory(current)?;
            let entry = entries
                .iter()
                .find(|e| {
                    e.attributes().is_directory()
                        && !e.is_dot()
                        && !e.is_dot_dot()
                        && e.short_name().eq_ignore_ascii_case(component)
                })
                .ok_or(Fat32Error::NotFound)?;

            current = entry.first_cluster();
        }

        Ok(current)
    }

    /// Séparer un chemin en dossier + nom de fichier
    fn parse_path(&mut self, path: &str) -> Result<(u32, &str)> {
        let (dir, name) = if let Some(pos) = path.rfind('/') {
            let (dir_path, name) = path.split_at(pos);
            (dir_path, &name[1..])
        } else {
            ("", path)
        };

        let dir_cluster = if dir.is_empty() {
            self.current_directory
        } else {
            self.resolve_path(dir)?
        };

        Ok((dir_cluster, name))
    }

    /// Trouver le dossier parent
    fn find_parent(&mut self, cluster: u32) -> Result<u32> {
        let entries = self.read_directory(cluster)?;

        for entry in entries {
            if entry.is_dot_dot() {
                let parent = entry.first_cluster();
                return Ok(if parent == 0 {
                    self.boot_sector.root_cluster
                } else {
                    parent
                });
            }
        }

        Err(Fat32Error::NotFound)
    }

    /// Lire toutes les entrées d'un répertoire
    fn read_directory(&mut self, cluster: u32) -> Result<Vec<DirectoryEntry>> {
        let mut fat = FatTable::new(&mut self.device, &self.boot_sector);
        let clusters = fat.cluster_chain(cluster)?;

        let mut entries = Vec::new();

        for cluster in clusters {
            let data = self.read_cluster(cluster)?;

            // Parser les entrées (32 octets chacune)
            for chunk in data.chunks_exact(DirectoryEntry::SIZE) {
                let entry = unsafe { DirectoryEntry::from_bytes(chunk) };

                if entry.is_end() {
                    return Ok(entries);
                }

                if entry.is_valid()
                    && !entry.attributes().is_long_name()
                    && !entry.attributes().is_volume_id()
                {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    /// Lire un cluster complet
    fn read_cluster(&mut self, cluster: u32) -> Result<Vec<u8>> {
        let first_sector = self.cluster_to_sector(cluster);
        let mut buffer = alloc::vec![0u8; self.boot_sector.cluster_size() as usize];

        for i in 0..self.boot_sector.sectors_per_cluster as u32 {
            let offset = i * self.boot_sector.bytes_per_sector as u32;
            self.device.read_sector(
                first_sector + i,
                &mut buffer[offset as usize..(offset + self.boot_sector.bytes_per_sector as u32) as usize],
            )?;
        }

        Ok(buffer)
    }

    /// Convertir un numéro de cluster en numéro de secteur
    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        ((cluster - 2) * self.boot_sector.sectors_per_cluster as u32)
            + self.boot_sector.first_data_sector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_filesystem_creation() {
        let mut device = MockDevice { data: vec![0; 1024 * 512] };
        device.data[66] = 0x29;
        device.data[11..13].copy_from_slice(&512u16.to_le_bytes());
        device.data[13] = 1;
        device.data[14..16].copy_from_slice(&32u16.to_le_bytes());
        device.data[16] = 2;
        device.data[36..40].copy_from_slice(&8u32.to_le_bytes());
        device.data[44..48].copy_from_slice(&2u32.to_le_bytes());

        let fs = Fat32FileSystem::new(device);
        assert!(fs.is_ok());
    }
}