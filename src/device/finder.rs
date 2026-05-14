use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;
use anyhow::{Result, Context};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Partition not found: {0}")]
    PartitionNotFound(String),
    #[error("Slot suffix not found")]
    SlotSuffixNotFound,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("System property error")]
    SystemPropertyError,
}

pub struct BlockDeviceFinder;

impl BlockDeviceFinder {
    pub fn new() -> Self {
        BlockDeviceFinder
    }
    
    /// Get the current slot suffix (e.g., "_a", "_b")
    pub fn get_slot_suffix(&self) -> Result<String> {
        // First try getprop
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
        
        // If getprop failed or returned empty, try alternative methods
        self.get_slot_suffix_from_alternative()
    }
    
    fn get_slot_suffix_from_alternative(&self) -> Result<String> {
        // Try to get slot from bootloader
        let output = Command::new("getprop")
            .arg("ro.boot.slot")
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let suffix = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                
                if !suffix.is_empty() {
                    return Ok(format!("_{}", suffix));  // Add underscore
                }
            }
        }
        
        // Check if we're in recovery
        let recovery_check = Command::new("getprop")
            .arg("ro.bootmode")
            .output();
            
        if let Ok(output) = recovery_check {
            if output.status.success() {
                let mode = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_lowercase();
                
                if mode.contains("recovery") {
                    // In recovery, slot detection might be different
                    // Try to read from kernel cmdline
                    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
                        for part in cmdline.split_whitespace() {
                            if part.starts_with("androidboot.slot_suffix=") {
                                if let Some(slot) = part.split('=').nth(1) {
                                    return Ok(slot.to_string());
                                }
                            } else if part.starts_with("androidboot.slot=") {
                                if let Some(slot) = part.split('=').nth(1) {
                                    return Ok(format!("_{}", slot));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // No slot suffix found
        Ok(String::new())
    }
    
    /// Find the actual device path for a partition
    pub fn find_partition(&self, partition_name: &str, slot_suffix: &str) -> Result<String> {
        // Try /dev/block/by-name first
        let by_name_dir = Path::new("/dev/block/by-name");
        
        if by_name_dir.exists() {
            // Try with slot suffix first
            if !slot_suffix.is_empty() {
                let target_name = format!("{}{}", partition_name, slot_suffix);
                let target_path = by_name_dir.join(&target_name);
                
                if target_path.exists() {
                    match self.resolve_symlink(&target_path) {
                        Ok(path) => return Ok(path),
                        Err(_) => (), // Continue to other methods
                    }
                }
            }
            
            // Try without suffix
            let target_path = by_name_dir.join(partition_name);
            if target_path.exists() {
                return self.resolve_symlink(&target_path);
            }
        }
        
        // Search in other common locations
        self.search_in_common_locations(partition_name, slot_suffix)
    }
    
    fn resolve_symlink(&self, path: &Path) -> Result<String> {
        match fs::read_link(path) {
            Ok(real_path) => {
                if real_path.is_absolute() {
                    Ok(real_path.to_string_lossy().to_string())
                } else {
                    // Make it absolute relative to the symlink's directory
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
        // Common block device directories
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
        
        // Last resort: search in /dev/block
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
