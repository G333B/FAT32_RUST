//error management


use core::fmt;

pub type Result<T> = core::result::Result<T, Fat32Error>;

/// errors
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