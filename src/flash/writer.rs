use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use thiserror::Error;
use anyhow::{Result, Context};
use bytesize::ByteSize;

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
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Flashing image: {}", image_path);
        println!("Target device: {}", device_path);
        
        // 获取镜像文件大小
        let file_size = std::fs::metadata(image_path)?.len();
        println!("Image size: {}", ByteSize(file_size));
        
        // 使用dd命令刷写
        println!("Starting flash operation with dd...");
        
        let status = Command::new("dd")
            .arg(format!("if={}", image_path))
            .arg(format!("of={}", device_path))
            .arg("bs=4M")
            .arg("conv=fsync")
            .arg("status=progress")
            .status()
            .context("Failed to execute dd command")?;
        
        if status.success() {
            // 同步文件系统
            let _ = Command::new("sync").status();
            println!("Flash completed successfully");
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
    pub fn dump_partition(&self, device_path: &str, output_path: &str) -> Result<()> {
        if !self.is_root()? {
            return Err(FlashError::PermissionDenied.into());
        }
        
        println!("Dumping partition: {}", device_path);
        println!("Output file: {}", output_path);
        
        // 获取设备大小
        if let Ok(size) = self.get_device_size(device_path) {
            println!("Device size: {}", ByteSize(size));
        } else {
            println!("Device size: unknown");
        }
        
        // 使用dd命令提取
        println!("Starting dump operation with dd...");
        
        // 执行dd命令并捕获输出
        let mut child = Command::new("dd")
            .arg(format!("if={}", device_path))
            .arg(format!("of={}", output_path))
            .arg("bs=4M")
            .arg("conv=fsync")
            .arg("status=progress")
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute dd command")?;
        
        // 读取stderr输出以显示进度
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            
            for line in reader.lines() {
                match line {
                    Ok(line_str) => {
                        // 解析并显示进度
                        if line_str.contains("bytes") && line_str.contains("copied") {
                            print!("{}\r", line_str);
                            std::io::stdout().flush()?;
                        }
                    }
                    Err(e) => {
                        println!("Error reading dd output: {}", e);
                        break;
                    }
                }
            }
        }
        
        let status = child.wait()?;
        
        if status.success() {
            // 同步文件系统
            let _ = Command::new("sync").status();
            
            // 验证输出文件大小
            if let Ok(metadata) = std::fs::metadata(output_path) {
                println!("\nSuccessfully dumped {} to {}", ByteSize(metadata.len()), output_path);
            } else {
                println!("\nDump completed");
            }
            
            Ok(())
        } else {
            Err(FlashError::DumpFailed("dd command failed".to_string()).into())
        }
    }
    
    /// 获取设备大小
    fn get_device_size(&self, device_path: &str) -> Result<u64> {
        use std::os::unix::io::AsRawFd;
        use libc;
        
        const BLKGETSIZE64: libc::c_ulong = 0x80081272;
        
        let file = std::fs::File::open(device_path)?;
        let fd = file.as_raw_fd();
        let mut size: u64 = 0;
        
        let result = unsafe {
            libc::ioctl(fd, BLKGETSIZE64 as libc::c_int, &mut size as *mut u64)
        };
        
        if result < 0 {
            return Err(anyhow::anyhow!("Failed to get device size"));
        }
        
        Ok(size)
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
