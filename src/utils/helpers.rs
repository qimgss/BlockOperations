use anyhow::{Result, Context};
use std::process::Command;

pub fn is_root() -> Result<bool> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .context("Failed to execute id command")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let uid_str = stdout.trim();
    Ok(uid_str == "0")
}

pub fn get_api_level() -> Result<i32> {
    let output = Command::new("getprop")
        .arg("ro.build.version.sdk")
        .output()
        .context("Failed to execute getprop command")?;
    
    if output.status.success() {
        let api_str = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        api_str.parse::<i32>()
            .with_context(|| format!("Failed to parse API level: {}", api_str))
    } else {
        Ok(0)  // Unknown
    }
}

pub fn has_ab_partitions() -> bool {
    let result = Command::new("getprop")
        .arg("ro.build.ab_update")
        .output();
    
    match result {
        Ok(output) if output.status.success() => {
            let value = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            value == "true" || value == "1"
        }
        _ => false,
    }
}
