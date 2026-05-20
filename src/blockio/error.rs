use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockIOError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid block size: {0}")]
    InvalidBlockSize(u64),
    
    #[error("Size mismatch: expected {0}, got {1}")]
    SizeMismatch(u64, u64),
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Operation cancelled")]
    Cancelled,
}
