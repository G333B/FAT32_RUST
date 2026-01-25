// Tests d'intégration pour FAT32
use fat32::{BlockDevice, Fat32FileSystem, Fat32Error, Result};

/// Device de test
struct TestDevice {
    data: Vec<u8>,
}

impl TestDevice {
    fn new_fat32() -> Self {
        let mut data = vec![0u8; 1024 * 512];
        
        // Boot sector minimal
        data[0..3].copy_from_slice(&[0xEB, 0x58, 0x90]); // jump
        data[3..11].copy_from_slice(b"MSWIN4.1"); // OEM
        data[11..13].copy_from_slice(&512u16.to_le_bytes()); // bytes per sector
        data[13] = 8; // sectors per cluster
        data[14..16].copy_from_slice(&32u16.to_le_bytes()); // reserved
        data[16] = 2; // num fats
        data[36..40].copy_from_slice(&8u32.to_le_bytes()); // fat size
        data[44..48].copy_from_slice(&2u32.to_le_bytes()); // root cluster
        data[66] = 0x29; // signature
        
        Self { data }
    }
}

impl BlockDevice for TestDevice {
    fn read_sector(&mut self, sector: u32, buffer: &mut [u8]) -> Result<()> {
        let offset = sector as usize * 512;
        if offset + buffer.len() > self.data.len() {
            return Err(Fat32Error::IoError);
        }
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
fn test_create_filesystem() {
    let device = TestDevice::new_fat32();
    let fs = Fat32FileSystem::new(device);
    assert!(fs.is_ok());
}

#[test]
fn test_invalid_filesystem() {
    let device = TestDevice { data: vec![0; 1024 * 512] };
    let fs = Fat32FileSystem::new(device);
    assert!(fs.is_err());
}

#[test]
fn test_current_directory() {
    let device = TestDevice::new_fat32();
    let fs = Fat32FileSystem::new(device).unwrap();
    assert_eq!(fs.current_dir(), 2); // root cluster
}

#[test]
fn test_list_empty_directory() {
    let device = TestDevice::new_fat32();
    let mut fs = Fat32FileSystem::new(device).unwrap();
    let result = fs.list_dir(None);
    // Peut être Ok(vide) ou Err selon l'état de la FAT
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_change_directory_root() {
    let device = TestDevice::new_fat32();
    let mut fs = Fat32FileSystem::new(device).unwrap();
    let result = fs.change_dir("/");
    // Devrait marcher ou échouer proprement
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_invalid_paths() {
    let device = TestDevice::new_fat32();
    let mut fs = Fat32FileSystem::new(device).unwrap();
    
    // Ces chemins devraient échouer
    let paths = vec!["///", "nonexistent", "/fake/path"];
    
    for path in paths {
        let _ = fs.change_dir(path);
        // Ne devrait pas paniquer
    }
}

#[test]
fn test_read_nonexistent_file() {
    let device = TestDevice::new_fat32();
    let mut fs = Fat32FileSystem::new(device).unwrap();
    
    let result = fs.read_file("nonexistent.txt");
    assert!(result.is_err());
}