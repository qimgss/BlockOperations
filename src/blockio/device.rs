use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::time::{Duration, Instant};
use crate::progress::ProgressDisplay;
use anyhow::{Result, Context};
use libc;

// BLKGETSIZE64 ioctl命令
const BLKGETSIZE64: libc::c_ulong = 0x80081272;

pub struct BlockDevice {
    file: File,
    path: String,
    block_size: u64,
    total_size: u64,
}

impl BlockDevice {
    pub fn open(path: &str) -> Result<Self> {
        println!("Opening device: {}", path);
        let file = File::open(path)
            .context(format!("Failed to open device: {}", path))?;
        
        // 获取块大小
        let block_size = Self::get_device_block_size(path)?;
        println!("Block size: {} bytes", block_size);
        
        // 尝试多种方法获取设备大小
        let total_size = match Self::get_device_size_ioctl(&file) {
            Ok(size) if size > 0 => {
                println!("Got device size via ioctl: {} bytes", size);
                size
            }
            _ => {
                // ioctl失败，尝试sysfs
                match Self::get_device_size_from_sysfs(path) {
                    Ok(size) if size > 0 => {
                        println!("Got device size from sysfs: {} bytes", size);
                        size
                    }
                    _ => {
                        // 尝试读取设备信息来确定大小
                        println!("Could not determine device size, will try to read...");
                        0
                    }
                }
            }
        };
        
        if total_size > 0 {
            println!("Device size: {} bytes", total_size);
        } else {
            println!("Device size: unknown (will read until EOF)");
        }
        
        Ok(Self {
            file,
            path: path.to_string(),
            block_size,
            total_size,
        })
    }
    
    fn get_device_size_ioctl(file: &File) -> Result<u64> {
        use std::os::unix::io::AsRawFd;
        
        let fd = file.as_raw_fd();
        let mut size: u64 = 0;
        
        // 使用libc::ioctl
        let result = unsafe {
            libc::ioctl(fd, BLKGETSIZE64 as libc::c_int, &mut size as *mut u64)
        };
        
        if result < 0 {
            let err = std::io::Error::last_os_error();
            println!("ioctl failed: {}", err);
            return Ok(0);
        }
        
        Ok(size)
    }
    
    fn get_device_size_from_sysfs(path: &str) -> Result<u64> {
        // 从设备路径提取设备名
        let dev_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if dev_name.is_empty() {
            return Ok(0);
        }
        
        // 尝试从sysfs获取大小
        let size_path = format!("/sys/class/block/{}/size", dev_name);
        
        match std::fs::read_to_string(&size_path) {
            Ok(content) => {
                let sectors = content.trim().parse::<u64>()
                    .context(format!("Failed to parse size from: {}", size_path))?;
                // 扇区数 * 512 = 字节数
                Ok(sectors * 512)
            }
            Err(e) => {
                println!("Failed to read {}: {}", size_path, e);
                Ok(0)
            }
        }
    }
    
    fn get_device_block_size(path: &str) -> Result<u64> {
        // 尝试从sysfs获取块大小
        let dev_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if dev_name.is_empty() {
            return Ok(512); // 默认值
        }
        
        let sysfs_path = format!("/sys/class/block/{}/queue/logical_block_size", dev_name);
        
        match std::fs::read_to_string(&sysfs_path) {
            Ok(content) => {
                match content.trim().parse::<u64>() {
                    Ok(size) => Ok(size),
                    Err(_) => Ok(512), // 解析失败，使用默认值
                }
            }
            Err(_) => Ok(512), // 读取失败，使用默认值
        }
    }
    
    pub fn read_to_file(&mut self, output_path: &str, progress: &dyn ProgressDisplay) -> Result<()> {
        use std::fs::OpenOptions;
        
        println!("Creating output file: {}", output_path);
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        
        // 使用较小的缓冲区，但必须是块大小的倍数
        let buffer_size = 64 * 1024; // 64KB缓冲区，块设备通常喜欢这个大小
        let mut buffer = vec![0u8; buffer_size];
        let mut total_read = 0u64;
        let start_time = Instant::now();
        
        println!("Seeking to start of device...");
        self.file.seek(SeekFrom::Start(0))?;
        
        if self.total_size > 0 {
            println!("Starting read operation for {} bytes...", self.total_size);
        } else {
            println!("Starting read operation (unknown size)...");
        }
        
        // 如果知道设备大小，使用已知大小的读取
        if self.total_size > 0 {
            let mut last_log_time = Instant::now();
            let mut bytes_since_last_log = 0u64;
            
            while total_read < self.total_size {
                // 计算剩余要读取的字节数
                let remaining = self.total_size - total_read;
                let bytes_to_read = std::cmp::min(remaining, buffer_size as u64) as usize;
                
                if bytes_to_read == 0 {
                    println!("No more data to read");
                    break;
                }
                
                // 调整缓冲区大小
                if buffer.len() != bytes_to_read {
                    buffer.resize(bytes_to_read, 0);
                }
                
                // 尝试读取，设置超时
                let read_result = Self::read_with_timeout(&mut self.file, &mut buffer[..bytes_to_read]);
                
                match read_result {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            println!("Reached end of device (EOF) at {} bytes", total_read);
                            break;
                        }
                        
                        // 写入到输出文件
                        output.write_all(&buffer[..bytes_read])?;
                        total_read += bytes_read as u64;
                        bytes_since_last_log += bytes_read as u64;
                        
                        // 更新进度
                        progress.update(bytes_read as u64);
                        
                        // 每秒记录一次进度，避免太多日志
                        if last_log_time.elapsed() >= Duration::from_secs(1) {
                            let percent = (total_read as f64 / self.total_size as f64 * 100.0) as u8;
                            let elapsed = start_time.elapsed();
                            let speed = if elapsed.as_secs() > 0 {
                                total_read as f64 / elapsed.as_secs_f64()
                            } else {
                                0.0
                            };
                            
                            println!("Progress: {}% ({} / {} bytes) - {:.2} MB/s", 
                                   percent, total_read, self.total_size, speed / 1024.0 / 1024.0);
                            
                            last_log_time = Instant::now();
                            bytes_since_last_log = 0;
                        }
                        
                        // 检查是否已经读取了所有数据
                        if total_read >= self.total_size {
                            println!("Read complete: {} bytes", total_read);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock || 
                           e.kind() == std::io::ErrorKind::TimedOut {
                            println!("Read timeout, retrying...");
                            continue;
                        }
                        println!("Read error at {} bytes: {}", total_read, e);
                        return Err(e.into());
                    }
                }
            }
        } else {
            // 未知大小，读取直到EOF
            println!("Reading with unknown size, will read until EOF...");
            let mut last_log_time = Instant::now();
            
            loop {
                // 尝试读取，设置超时
                let read_result = Self::read_with_timeout(&mut self.file, &mut buffer);
                
                match read_result {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            println!("Reached end of device (EOF) at {} bytes", total_read);
                            break;
                        }
                        
                        // 写入到输出文件
                        output.write_all(&buffer[..bytes_read])?;
                        total_read += bytes_read as u64;
                        
                        // 更新进度
                        progress.update(bytes_read as u64);
                        
                        // 每秒记录一次进度
                        if last_log_time.elapsed() >= Duration::from_secs(1) {
                            let elapsed = start_time.elapsed();
                            let speed = if elapsed.as_secs() > 0 {
                                total_read as f64 / elapsed.as_secs_f64()
                            } else {
                                0.0
                            };
                            
                            println!("Read: {} bytes - {:.2} MB/s", 
                                   total_read, speed / 1024.0 / 1024.0);
                            
                            last_log_time = Instant::now();
                        }
                        
                        // 限制最大读取大小（防止无限读取）
                        if total_read > 1024 * 1024 * 1024 { // 1GB
                            println!("Reached safety limit of 1GB, stopping");
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock || 
                           e.kind() == std::io::ErrorKind::TimedOut {
                            println!("Read timeout, retrying...");
                            continue;
                        } else if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            println!("Unexpected EOF at {} bytes", total_read);
                            break;
                        } else {
                            println!("Read error at {} bytes: {}", total_read, e);
                            return Err(e.into());
                        }
                    }
                }
            }
            
            // 更新总大小
            self.total_size = total_read;
        }
        
        println!("Syncing output file...");
        output.sync_all()?;
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_read as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("Read completed: {} bytes written to {} in {:.2}s ({:.2} MB/s)", 
               total_read, output_path, elapsed.as_secs_f32(), speed / 1024.0 / 1024.0);
        
        // 验证输出文件大小
        if let Ok(metadata) = std::fs::metadata(output_path) {
            let output_size = metadata.len();
            println!("Output file size: {} bytes", output_size);
            if output_size != total_read {
                println!("Warning: Output file size ({}) doesn't match bytes read ({})", 
                       output_size, total_read);
            }
        }
        
        Ok(())
    }
    
    // 带超时的读取函数
    fn read_with_timeout(file: &mut File, buffer: &mut [u8]) -> std::io::Result<usize> {
        use std::os::unix::io::AsRawFd;
        
        let fd = file.as_raw_fd();
        
        // 设置读取超时
        unsafe {
            let mut timeout = libc::timeval {
                tv_sec: 5,  // 5秒超时
                tv_usec: 0,
            };
            
            if libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_RCVTIMEO, 
                              &timeout as *const _ as *const libc::c_void, 
                              std::mem::size_of_val(&timeout) as libc::socklen_t) < 0 {
                // 设置超时失败，继续普通读取
            }
        }
        
        // 普通读取
        file.read(buffer)
    }
    
    pub fn write_from_file(&mut self, input_path: &str, progress: &dyn ProgressDisplay) -> Result<()> {
        use std::fs::File;
        use std::io::Read;
        
        println!("Opening input file: {}", input_path);
        let mut input = File::open(input_path)
            .context(format!("Failed to open input file: {}", input_path))?;
        
        let file_size = input.metadata()?.len();
        println!("Input file size: {} bytes", file_size);
        
        let buffer_size = 64 * 1024; // 64KB缓冲区
        let mut buffer = vec![0u8; buffer_size];
        let mut total_written = 0u64;
        let start_time = Instant::now();
        
        println!("Seeking to start of device...");
        self.file.seek(SeekFrom::Start(0))?;
        
        println!("Starting write operation...");
        
        loop {
            // 计算本次要写入的字节数
            let bytes_to_read = if file_size - total_written < buffer_size as u64 {
                (file_size - total_written) as usize
            } else {
                buffer_size
            };
            
            if bytes_to_read == 0 {
                println!("No more data to write");
                break;
            }
            
            // 调整缓冲区大小
            if buffer.len() != bytes_to_read {
                buffer.resize(bytes_to_read, 0);
            }
            
            // 从输入文件读取
            match input.read(&mut buffer[..bytes_to_read]) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        println!("Reached end of input file (EOF)");
                        break;
                    }
                    
                    // 写入到块设备
                    self.file.write_all_at(&buffer[..bytes_read], total_written)?;
                    total_written += bytes_read as u64;
                    
                    // 更新进度
                    progress.update(bytes_read as u64);
                    
                    // 每秒记录一次进度
                    if start_time.elapsed().as_secs() >= 1 && 
                       total_written % (10 * 1024 * 1024) < bytes_read as u64 {
                        let percent = (total_written as f64 / file_size as f64 * 100.0) as u8;
                        let elapsed = start_time.elapsed();
                        let speed = if elapsed.as_secs() > 0 {
                            total_written as f64 / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };
                        
                        println!("Progress: {}% ({} / {} bytes) - {:.2} MB/s", 
                               percent, total_written, file_size, speed / 1024.0 / 1024.0);
                    }
                    
                    // 检查是否已经写入了所有数据
                    if total_written >= file_size {
                        println!("Write complete: {} bytes", total_written);
                        break;
                    }
                }
                Err(e) => {
                    println!("Write error: {}, bytes written so far: {}", e, total_written);
                    return Err(e.into());
                }
            }
        }
        
        println!("Syncing device...");
        self.file.sync_all()?;
        
        // 额外的sync命令确保数据写入
        println!("Running sync command...");
        let _ = std::process::Command::new("sync").status();
        
        let elapsed = start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            total_written as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        println!("Write completed successfully: {} bytes written to {} in {:.2}s ({:.2} MB/s)", 
               total_written, self.path, elapsed.as_secs_f32(), speed / 1024.0 / 1024.0);
        
        Ok(())
    }
    
    pub fn get_size(&self) -> u64 {
        self.total_size
    }
    
    pub fn block_size(&self) -> u64 {
        self.block_size
    }
    
    pub fn get_path(&self) -> &str {
        &self.path
    }
}
