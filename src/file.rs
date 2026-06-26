use std::fs::File;
use anyhow::{Result, Context};
use super::device::BlockDevice;

pub struct ImageFile {
    file: File,
    path: String,
    size: u64,
}

impl ImageFile {
    pub fn open(path: &str) -> Result<Self> {
        let file = File::open(path)
            .context(format!("Failed to open image file: {}", path))?;
        
        let size = file.metadata()?.len();
        
        Ok(Self {
            file,
            path: path.to_string(),
            size,
        })
    }
    
    pub fn create(path: &str) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .context(format!("Failed to create image file: {}", path))?;
        
        Ok(Self {
            file,
            path: path.to_string(),
            size: 0,
        })
    }
    
    pub fn get_size(&self) -> u64 {
        self.size
    }
    
    pub fn verify(&self, device: &BlockDevice) -> Result<bool> {
        if self.size != device.get_size() {
            return Ok(false);
        }
        Ok(true)
    }
}
