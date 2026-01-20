// CLI pour le filesystem FAT32
use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::process;

use fat32::{BlockDevice, Fat32FileSystem, Fat32Error, Result};

/// Device basé sur un fichier
struct FileDevice {
    file: File,
}

impl FileDevice {
    fn open(path: &str) -> io::Result<Self> {
        let file = File::options().read(true).write(true).open(path)?;
        Ok(Self { file })
    }
}

impl BlockDevice for FileDevice {
    fn read_sector(&mut self, sector: u32, buffer: &mut [u8]) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(sector as u64 * 512))
            .map_err(|_| Fat32Error::IoError)?;
        self.file
            .read_exact(buffer)
            .map_err(|_| Fat32Error::IoError)?;
        Ok(())
    }

    fn write_sector(&mut self, sector: u32, buffer: &[u8]) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(sector as u64 * 512))
            .map_err(|_| Fat32Error::IoError)?;
        self.file
            .write_all(buffer)
            .map_err(|_| Fat32Error::IoError)?;
        Ok(())
    }

    fn sector_size(&self) -> usize {
        512
    }
}

fn print_help(program: &str) {
    println!("FAT32 Filesystem");
    println!();
    println!("Usage: {} <image> <commande> [args]", program);
    println!();
    println!("Commandes:");
    println!("  ls [chemin]      Liste les fichiers");
    println!("  cat <fichier>    Affiche un fichier");
    println!("  cd <chemin>      Change de dossier");
    println!("  pwd              Affiche le dossier courant");
    println!();
    println!("Exemples:");
    println!("  {} disk.img ls", program);
    println!("  {} disk.img cat /readme.txt", program);
    println!("  {} disk.img cd /dossier", program);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help(&args[0]);
        return Ok(()); // ← Au lieu de process::exit(1)
    }

    let image_path = &args[1];

    // Ouvrir l'image
    let device = match FileDevice::open(image_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Erreur: impossible d'ouvrir '{}': {}", image_path, e);
            process::exit(1);
        }
    };

    // Créer le filesystem
    let mut fs = match Fat32FileSystem::new(device) {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("Erreur: filesystem invalide: {}", e);
            process::exit(1);
        }
    };

    // Commande par défaut = ls
    let cmd = args.get(2).map(|s| s.as_str()).unwrap_or("ls");

    let result = match cmd {
        "ls" => {
            let path = args.get(3).map(|s| s.as_str());
            match fs.list_dir(path) {
                Ok(entries) => {
                    if entries.is_empty() {
                        println!("(vide)");
                    } else {
                        for entry in entries {
                            let type_str = if entry.attributes().is_directory() {
                                "DIR "
                            } else {
                                "FILE"
                            };
                            println!("{} {:>10}  {}", 
                                type_str, 
                                entry.file_size(), 
                                entry.short_name()
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        
        "cat" | "more" => {
            if let Some(file) = args.get(3) {
                match fs.read_file(file) {
                    Ok(data) => {
                        io::stdout().write_all(&data).map_err(|_| Fat32Error::IoError)?;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                eprintln!("Usage: {} {} cat <fichier>", args[0], args[1]);
                process::exit(1);
            }
        }
        
        "cd" => {
            if let Some(path) = args.get(3) {
                fs.change_dir(path)?;
                println!("Dossier changé: {}", path);
                println!("Cluster: {}", fs.current_dir());
                Ok(())
            } else {
                eprintln!("Usage: {} {} cd <chemin>", args[0], args[1]);
                process::exit(1);
            }
        }
        
        "pwd" => {
            println!("Cluster du répertoire courant: {}", fs.current_dir());
            Ok(())
        }
        
        _ => {
            eprintln!("Commande inconnue: {}", cmd);
            print_help(&args[0]);
            process::exit(1);
        }
    };

   if let Err(e) = result {
        eprintln!("Erreur: {}", e);
        return Err(*Box::new(e)); // ← Retourner l'erreur
    }

    Ok(())
}