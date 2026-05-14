use std::process::Command;
use thiserror::Error;
use anyhow::{Result, Context};

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("Failed to open file: {0}")]
    FileOpenError(#[from] std::io::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Flash failed: {0}")]
    FlashFailed(String),
    #[error("Dump failed: {0}")]
    DumpFailed(String),
}

pub struct ImageFlasher;

impl ImageFlasher {
    pub fn new() -> Self {
        ImageFlasher
    }
    
    /// Flash an image to a block device
    pub fn flash_image(&self, image_path: &str, device_path: &str) -> Result<()> {
        // Check if we have root permissions
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Using dd to flash image...");
        let status = Command::new("dd")
            .arg(format!("if={}", image_path))
            .arg(format!("of={}", device_path))
            .arg("bs=4M")
            .arg("conv=fsync")
            .status()
            .context("Failed to execute dd command")?;
        
        if status.success() {
            // Force sync
            let _ = Command::new("sync").status();
            Ok(())
        } else {
            Err(FlashError::FlashFailed("dd command failed".to_string()).into())
        }
    }
    
    fn is_root(&self) -> Result<bool> {
        let output = Command::new("id")
            .arg("-u")
            .output()
            .context("Failed to execute id command")?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let uid_str = stdout.trim();
        Ok(uid_str == "0")
    }
}

pub struct ImageDumper;

impl ImageDumper {
    pub fn new() -> Self {
        ImageDumper
    }
    
    /// Dump a partition to an image file
    pub fn dump_partition(&self, device_path: &str, output_path: &str, block_size: &str, count: Option<&str>) -> Result<()> {
        // Check if we have root permissions
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Using dd to dump partition...");
        
        // Build dd command with correct argument format
        let mut cmd = Command::new("dd");
        cmd.arg(format!("if={}", device_path))
           .arg(format!("of={}", output_path))
           .arg(format!("bs={}", block_size))
           .arg("conv=fsync");
        
        // Add count if specified
        if let Some(count_str) = count {
            cmd.arg(format!("count={}", count_str));
        }
        
        // Add status=progress for better user feedback
        cmd.arg("status=progress");
        
        // Execute the command
        let status = cmd.status()
            .context("Failed to execute dd command")?;
        
        if status.success() {
            // Force sync
            let _ = Command::new("sync").status();
            Ok(())
        } else {
            // Get error output for better debugging
            let output = Command::new("dd")
                .arg(format!("if={}", device_path))
                .arg(format!("of={}", output_path))
                .arg(format!("bs={}", block_size))
                .arg("conv=fsync")
                .output();
            
            match output {
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(FlashError::DumpFailed(format!("dd command failed: {}", stderr)).into())
                }
                Err(e) => {
                    Err(FlashError::DumpFailed(format!("dd command failed: {}", e)).into())
                }
            }
        }
    }
    
    /// Alternative dump method with simpler command
    pub fn dump_partition_simple(&self, device_path: &str, output_path: &str) -> Result<()> {
        println!("Using dd to dump partition (simple method)...");
        
        let command = format!("dd if={} of={} bs=4M conv=fsync status=progress", device_path, output_path);
        println!("Executing: {}", command);
        
        let status = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .status()
            .context("Failed to execute dd command via shell")?;
        
        if status.success() {
            let _ = Command::new("sync").status();
            Ok(())
        } else {
            Err(FlashError::DumpFailed("dd command failed".to_string()).into())
        }
    }
    
    fn is_root(&self) -> Result<bool> {
        let output = Command::new("id")
            .arg("-u")
            .output()
            .context("Failed to execute id command")?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let uid_str = stdout.trim();
        Ok(uid_str == "0")
    }
}
