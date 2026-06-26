use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use std::time::Instant;
use thiserror::Error;
use anyhow::{Result, Context};
use bytesize::ByteSize;
use crate::blockdev::BlockDevice;

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("Failed to open file: {0}")]
    FileOpenError(#[from] std::io::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Flash failed: {0}")]
    FlashFailed(String),
    // 删除 DumpFailed，因为现在只有一个 ImageDumper
}

pub struct ImageFlasher;

impl ImageFlasher {
    pub fn new() -> Self {
        ImageFlasher
    }
    
    pub fn flash_image(&self, image_path: &str, device_path: &str) -> Result<()> {
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Opening block device: {}", device_path);
        let device = BlockDevice::open(device_path)
            .context(format!("Failed to open block device: {}", device_path))?;
        
        println!("Opening image: {}", image_path);
        let mut image = File::open(image_path)
            .context(format!("Failed to open image: {}", image_path))?;
        
        let image_size = image.metadata()?.len();
        println!("Image size: {}", ByteSize(image_size));
        println!("Device size: {}", ByteSize(device.size()));
        
        if image_size > device.size() {
            return Err(FlashError::FlashFailed(
                format!("Image too large: {} > {}", image_size, device.size())
            ).into());
        }
        
        let buffer_size = 4 * 1024 * 1024;
        let mut buffer = vec![0u8; buffer_size];
        let mut total_written = 0u64;
        let start_time = Instant::now();
        
        println!("Starting flash operation...");
        
        while total_written < image_size {
            let to_read = std::cmp::min(buffer_size as u64, image_size - total_written) as usize;
            
            let bytes_read = image.read(&mut buffer[..to_read])?;
            if bytes_read == 0 { break; }
            
            let bytes_written = device.write(&buffer[..bytes_read], total_written)?;
            if bytes_written != bytes_read {
                return Err(FlashError::FlashFailed(
                    format!("Write mismatch: wrote {} of {} bytes", bytes_written, bytes_read)
                ).into());
            }
            
            total_written += bytes_written as u64;
            
            if total_written % (10 * 1024 * 1024) < bytes_written as u64 {
                let percent = (total_written as f64 / image_size as f64 * 100.0) as u8;
                let elapsed = start_time.elapsed();
                let speed = if elapsed.as_secs() >= 1 {
                    total_written as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                
                print!("\rFlashing: {}% ({}/{}) - {:.2} MB/s", 
                       percent, ByteSize(total_written), ByteSize(image_size),
                       speed / 1024.0 / 1024.0);
                std::io::stdout().flush()?;
            }
        }
        
        println!("\nFlushing device cache...");
        device.flush()?;
        
        println!("Skipping reread partition table (not applicable for partition devices)");
        
        let _ = Command::new("sync").status();
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() >= 1 {
            total_written as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("\n✅ Flash completed: {} written in {:.2}s ({:.2} MB/s)", 
                 ByteSize(total_written), elapsed.as_secs_f32(), 
                 speed / 1024.0 / 1024.0);
        
        Ok(())
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
    
    pub fn dump_partition(&self, device_path: &str, output_path: &str) -> Result<()> {
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Opening block device: {}", device_path);
        let device = BlockDevice::open(device_path)
            .context(format!("Failed to open block device: {}", device_path))?;
        
        println!("Creating output file: {}", output_path);
        let mut output = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        
        let device_size = device.size();
        println!("Device size: {}", ByteSize(device_size));
        
        let buffer_size = 4 * 1024 * 1024;
        let mut buffer = vec![0u8; buffer_size];
        let mut total_read = 0u64;
        let start_time = Instant::now();
        
        println!("Starting dump operation...");
        
        while total_read < device_size {
            let to_read = std::cmp::min(buffer_size as u64, device_size - total_read) as usize;
            
            let bytes_read = device.read(&mut buffer[..to_read], total_read)?;
            if bytes_read == 0 { break; }
            
            output.write_all(&buffer[..bytes_read])?;
            total_read += bytes_read as u64;
            
            if total_read % (10 * 1024 * 1024) < bytes_read as u64 {
                let percent = (total_read as f64 / device_size as f64 * 100.0) as u8;
                let elapsed = start_time.elapsed();
                let speed = if elapsed.as_secs() >= 1 {
                    total_read as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                
                print!("\rDumping: {}% ({}/{}) - {:.2} MB/s", 
                       percent, ByteSize(total_read), ByteSize(device_size),
                       speed / 1024.0 / 1024.0);
                std::io::stdout().flush()?;
            }
        }
        
        output.sync_all()?;
        let _ = Command::new("sync").status();
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() >= 1 {
            total_read as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("\n✅ Dump completed: {} written in {:.2}s ({:.2} MB/s)", 
                 ByteSize(total_read), elapsed.as_secs_f32(), 
                 speed / 1024.0 / 1024.0);
        
        Ok(())
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