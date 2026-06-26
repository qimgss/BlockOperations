use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;
use anyhow::{Result, Context};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Partition not found: {0}")]
    PartitionNotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    // 删除未使用的变体
}

pub struct BlockDeviceFinder;

impl BlockDeviceFinder {
    pub fn new() -> Self {
        BlockDeviceFinder
    }
    
    pub fn get_slot_suffix(&self) -> Result<String> {
        let output = Command::new("getprop")
            .arg("ro.boot.slot_suffix")
            .output()
            .context("Failed to execute getprop command")?;
            
        if output.status.success() {
            let suffix = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            
            if !suffix.is_empty() {
                return Ok(suffix);
            }
        }
        
        self.get_slot_suffix_from_alternative()
    }
    
    fn get_slot_suffix_from_alternative(&self) -> Result<String> {
        let output = Command::new("getprop")
            .arg("ro.boot.slot")
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let suffix = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                
                if !suffix.is_empty() {
                    return Ok(format!("_{}", suffix));
                }
            }
        }
        
        Ok(String::new())
    }
    
    pub fn find_partition(&self, partition_name: &str, slot_suffix: &str) -> Result<String> {
        let by_name_dir = Path::new("/dev/block/by-name");
        
        if by_name_dir.exists() {
            if !slot_suffix.is_empty() {
                let target_name = format!("{}{}", partition_name, slot_suffix);
                let target_path = by_name_dir.join(&target_name);
                
                if target_path.exists() {
                    match self.resolve_symlink(&target_path) {
                        Ok(path) => return Ok(path),
                        Err(_) => (),
                    }
                }
            }
            
            let target_path = by_name_dir.join(partition_name);
            if target_path.exists() {
                return self.resolve_symlink(&target_path);
            }
        }
        
        self.search_in_common_locations(partition_name, slot_suffix)
    }
    
    fn resolve_symlink(&self, path: &Path) -> Result<String> {
        match fs::read_link(path) {
            Ok(real_path) => {
                if real_path.is_absolute() {
                    Ok(real_path.to_string_lossy().to_string())
                } else {
                    let parent = path.parent().unwrap_or_else(|| Path::new("/"));
                    let absolute = parent.join(real_path);
                    match absolute.canonicalize() {
                        Ok(abs_path) => Ok(abs_path.to_string_lossy().to_string()),
                        Err(_) => Ok(absolute.to_string_lossy().to_string()),
                    }
                }
            }
            Err(e) => Err(e.into()),
        }
    }
    
    fn search_in_common_locations(&self, partition_name: &str, slot_suffix: &str) -> Result<String> {
        let search_dirs = [
            "/dev/block/platform",
            "/dev/block/bootdevice/by-name",
            "/dev/block/mapper",
        ];
        
        let target_names = if !slot_suffix.is_empty() {
            vec![
                format!("{}{}", partition_name, slot_suffix),
                partition_name.to_string(),
            ]
        } else {
            vec![partition_name.to_string()]
        };
        
        for dir in &search_dirs {
            let dir_path = Path::new(dir);
            if dir_path.exists() {
                if let Ok(entries) = fs::read_dir(dir_path) {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            for target in &target_names {
                                if name_str == *target || name_str.contains(target) {
                                    return self.resolve_symlink(&path)
                                        .or_else(|_| Ok(path.to_string_lossy().to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        self.search_in_dev_block(partition_name)
    }
    
    fn search_in_dev_block(&self, partition_name: &str) -> Result<String> {
        let dev_block_dir = Path::new("/dev/block");
        
        if !dev_block_dir.exists() {
            return Err(DeviceError::PartitionNotFound(partition_name.to_string()).into());
        }
        
        if let Ok(entries) = fs::read_dir(dev_block_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.contains(partition_name) {
                        return Ok(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Err(DeviceError::PartitionNotFound(partition_name.to_string()).into())
    }
}