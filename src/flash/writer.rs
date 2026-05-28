use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::os::unix::fs::FileExt;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use thiserror::Error;
use anyhow::{Result, Context};
use bytesize::ByteSize;
use libc;

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

// BLKGETSIZE64 ioctl command
const BLKGETSIZE64: libc::c_ulong = 0x80081272;

/// Get block device size using ioctl
fn get_block_device_size(file: &File) -> Result<u64> {
    let fd = file.as_raw_fd();
    let mut size: u64 = 0;
    
    let result = unsafe {
        libc::ioctl(fd, BLKGETSIZE64 as libc::c_int, &mut size as *mut u64)
    };
    
    if result < 0 {
        return Err(anyhow::anyhow!("Failed to get device size: {}", std::io::Error::last_os_error()));
    }
    
    Ok(size)
}

pub struct ImageFlasher;

impl ImageFlasher {
    pub fn new() -> Self {
        ImageFlasher
    }
    
    /// Flash an image to a block device using pure Rust
    pub fn flash_image(&self, image_path: &str, device_path: &str) -> Result<()> {
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Opening device: {}", device_path);
        let mut device = File::open(device_path)
            .context(format!("Failed to open device: {}", device_path))?;
        
        println!("Opening image: {}", image_path);
        let mut image = File::open(image_path)
            .context(format!("Failed to open image: {}", image_path))?;
        
        // Get image size
        let image_size = image.metadata()?.len();
        println!("Image size: {}", ByteSize(image_size));
        
        // Get device size
        let device_size = get_block_device_size(&device)?;
        println!("Device size: {}", ByteSize(device_size));
        
        if image_size > device_size {
            return Err(FlashError::FlashFailed(
                format!("Image size ({} bytes) exceeds device size ({} bytes)", 
                         image_size, device_size)
            ).into());
        }
        
        // Seek to beginning of device
        device.seek(SeekFrom::Start(0))?;
        
        // Use 4MB buffer for efficient copying
        let buffer_size = 4 * 1024 * 1024; // 4MB
        let mut buffer = vec![0u8; buffer_size];
        let mut total_written = 0u64;
        
        println!("Starting flash operation...");
        let start_time = std::time::Instant::now();
        
        while total_written < image_size {
            // Calculate how much to read
            let remaining = image_size - total_written;
            let to_read = std::cmp::min(remaining, buffer_size as u64) as usize;
            
            // Read from image
            let bytes_read = image.read(&mut buffer[..to_read])?;
            if bytes_read == 0 {
                break;
            }
            
            // Write to device
            device.write_all(&buffer[..bytes_read])?;
            total_written += bytes_read as u64;
            
            // Show progress every 10MB
            if total_written % (10 * 1024 * 1024) < bytes_read as u64 {
                let percent = (total_written as f64 / image_size as f64 * 100.0) as u8;
                let elapsed = start_time.elapsed();
                let speed = if elapsed.as_secs() > 0 {
                    total_written as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                
                print!("\rFlashing: {}% ({}/{}) - {:.2} MB/s", 
                       percent, 
                       ByteSize(total_written), 
                       ByteSize(image_size),
                       speed / 1024.0 / 1024.0);
                std::io::stdout().flush()?;
            }
        }
        
        // Sync device
        device.sync_all()?;
        
        // Run system sync
        let _ = Command::new("sync").status();
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_written as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("\nFlash completed: {} written in {:.2}s ({:.2} MB/s)", 
                 ByteSize(total_written), 
                 elapsed.as_secs_f32(),
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
    
    /// Dump a partition to an image file using pure Rust
    pub fn dump_partition(&self, device_path: &str, output_path: &str) -> Result<()> {
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Opening device: {}", device_path);
        let mut device = File::open(device_path)
            .context(format!("Failed to open device: {}", device_path))?;
        
        println!("Creating output file: {}", output_path);
        let mut output = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        
        // Get device size
        let device_size = get_block_device_size(&device)?;
        println!("Device size: {}", ByteSize(device_size));
        
        if device_size == 0 {
            return Err(FlashError::DumpFailed("Device size is 0".to_string()).into());
        }
        
        // Seek to beginning of device
        device.seek(SeekFrom::Start(0))?;
        
        // Use 4MB buffer for efficient copying
        let buffer_size = 4 * 1024 * 1024; // 4MB
        let mut buffer = vec![0u8; buffer_size];
        let mut total_read = 0u64;
        
        println!("Starting dump operation...");
        let start_time = std::time::Instant::now();
        
        while total_read < device_size {
            // Calculate how much to read
            let remaining = device_size - total_read;
            let to_read = std::cmp::min(remaining, buffer_size as u64) as usize;
            
            // Read from device
            let bytes_read = device.read(&mut buffer[..to_read])?;
            if bytes_read == 0 {
                break;
            }
            
            // Write to output file
            output.write_all(&buffer[..bytes_read])?;
            total_read += bytes_read as u64;
            
            // Show progress every 10MB
            if total_read % (10 * 1024 * 1024) < bytes_read as u64 {
                let percent = (total_read as f64 / device_size as f64 * 100.0) as u8;
                let elapsed = start_time.elapsed();
                let speed = if elapsed.as_secs() > 0 {
                    total_read as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                
                print!("\rDumping: {}% ({}/{}) - {:.2} MB/s", 
                       percent, 
                       ByteSize(total_read), 
                       ByteSize(device_size),
                       speed / 1024.0 / 1024.0);
                std::io::stdout().flush()?;
            }
        }
        
        // Sync output file
        output.sync_all()?;
        
        // Run system sync
        let _ = Command::new("sync").status();
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_read as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("\nDump completed: {} written in {:.2}s ({:.2} MB/s)", 
                 ByteSize(total_read), 
                 elapsed.as_secs_f32(),
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