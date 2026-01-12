use core::fmt;

pub type Result<T> = core::result::Result<T, Fat32Error>;

/// Les différentes erreurs possibles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fat32Error {
    InvalidBootSector,
    InvalidCluster,
    InvalidPath,
    NotFound,
    NotADirectory,
    EndOfChain,
    IoError,
    BufferTooSmall,
    InvalidEntry,
}

impl fmt::Display for Fat32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidBootSector => write!(f, "Boot sector invalide"),
            Self::InvalidCluster => write!(f, "Numéro de cluster invalide"),
            Self::InvalidPath => write!(f, "Chemin invalide"),
            Self::NotFound => write!(f, "Fichier ou dossier non trouvé"),
            Self::NotADirectory => write!(f, "Ce n'est pas un dossier"),
            Self::EndOfChain => write!(f, "Fin de la chaîne"),
            Self::IoError => write!(f, "Erreur d'entrée/sortie"),
            Self::BufferTooSmall => write!(f, "Buffer trop petit"),
            Self::InvalidEntry => write!(f, "Entrée invalide"),
        }
    }
}