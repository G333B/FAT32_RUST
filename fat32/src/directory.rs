//! Gestion des entrées de répertoire

use alloc::string::String;
use core::fmt;

/// Attributs d'un fichier/dossier
#[derive(Copy, Clone)]
pub struct FileAttributes(pub u8);

impl FileAttributes {
    pub const READ_ONLY: u8 = 0x01;
    pub const HIDDEN: u8 = 0x02;
    pub const SYSTEM: u8 = 0x04;
    pub const VOLUME_ID: u8 = 0x08;
    pub const DIRECTORY: u8 = 0x10;
    pub const ARCHIVE: u8 = 0x20;
    pub const LONG_NAME: u8 = 0x0F;

    pub fn is_directory(&self) -> bool {
        self.0 & Self::DIRECTORY != 0
    }

    pub fn is_long_name(&self) -> bool {
        self.0 & Self::LONG_NAME == Self::LONG_NAME
    }

    pub fn is_volume_id(&self) -> bool {
        self.0 & Self::VOLUME_ID != 0
    }
}

impl fmt::Debug for FileAttributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attributes(0x{:02x})", self.0)
    }
}

/// Entrée de répertoire (32 octets)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attributes: u8,
    nt_reserved: u8,
    creation_time_tenth: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high: u16,
    write_time: u16,
    write_date: u16,
    first_cluster_low: u16,
    file_size: u32,
}

impl DirectoryEntry {
    pub const SIZE: usize = 32;

    /// Lire une entrée depuis des données brutes
    /// 
    /// # Safety
    /// 
    /// Cette fonction lit une structure packed depuis la mémoire.
    /// Le layout exact est défini par la spec FAT32.
    /// 
    /// Conditions requises :
    /// - `data` doit contenir au moins 32 octets
    /// - `data` doit représenter une entrée de répertoire valide
    pub unsafe fn from_bytes(data: &[u8]) -> Self {
        unsafe { core::ptr::read_unaligned(data.as_ptr() as *const DirectoryEntry) }
    }

    pub fn is_free(&self) -> bool {
        self.name[0] == 0xE5
    }

    
    pub fn is_end(&self) -> bool {
        self.name[0] == 0x00
    }


    pub fn is_valid(&self) -> bool {
        !self.is_free() && !self.is_end()
    }

    pub fn attributes(&self) -> FileAttributes {
        FileAttributes(self.attributes)
    }

    /// premier cluster
    pub fn first_cluster(&self) -> u32 {
        let high = self.first_cluster_high;
        let low = self.first_cluster_low;
        ((high as u32) << 16) | (low as u32)
    }

    /// Taille du fichier
    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    /// Convertir le nom en String lisible
    pub fn short_name(&self) -> String {
        let name_bytes = self.name;
        
        // Nom (8 caractères)
        let name_part = core::str::from_utf8(&name_bytes[..8])
            .unwrap_or("")
            .trim_end();

        // Extension (3 caractères)
        let ext_part = core::str::from_utf8(&name_bytes[8..11])
            .unwrap_or("")
            .trim_end();

        if ext_part.is_empty() {
            alloc::format!("{}", name_part)
        } else {
            alloc::format!("{}.{}", name_part, ext_part)
        }
    }

    /// Entrée "."
    pub fn is_dot(&self) -> bool {
        self.name[0] == b'.' && self.name[1] == b' '
    }

    /// Entrée ".."
    pub fn is_dot_dot(&self) -> bool {
        self.name[0] == b'.' && self.name[1] == b'.' && self.name[2] == b' '
    }
}

impl fmt::Debug for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Copiervaleurs au lieu de créer références
        let name = self.short_name();
        let attrs = self.attributes();
        let cluster = self.first_cluster();
        let size = self.file_size();
        
        f.debug_struct("DirectoryEntry")
            .field("name", &name)
            .field("attributes", &attrs)
            .field("cluster", &cluster)
            .field("size", &size)
            .finish()
    }
}

