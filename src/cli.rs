use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "blkops",
    author,
    version,
    about = "Android block device operation utility",
    long_about = "A CLI tool for finding, flashing, and dumping Android block devices"
)]
pub struct Cli {
    /// Search for a partition and show its device path
    #[arg(short = 's', long = "search", value_name = "PARTITION")]
    pub search: Option<String>,
    
    /// Flash an image to a partition
    #[arg(short = 'f', long = "flash", value_name = "IMAGE", num_args = 2, value_names = &["IMAGE", "PARTITION"])]
    pub flash: Option<Vec<String>>,
    
    /// Dump (extract) a partition to an image file
    #[arg(short = 'd', long = "dump", value_name = "PARTITION", num_args = 2, value_names = &["PARTITION", "IMAGE"])]
    pub dump: Option<Vec<String>>,
    
    /// Block size for dump/flash commands (default: 4M)
    #[arg(short = 'b', long = "block-size", default_value = "4M")]
    pub block_size: Option<String>,
    
    /// Number of blocks to copy (for dump command)
    #[arg(short = 'c', long = "count")]
    pub count: Option<String>,
    
    /// Show help information
    #[arg(short = 'h', long = "help")]
    pub help: bool,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
    
    pub fn should_show_help(&self) -> bool {
        self.help || (self.search.is_none() && self.flash.is_none() && self.dump.is_none())
    }
}

pub fn print_help() {
    println!("blkops - Android Block Device Utility");
    println!();
    println!("Usage:");
    println!("  blkops -s <partition>              Search for a partition and show its device path");
    println!("  blkops -f <image> <partition>      Flash image to partition");
    println!("  blkops -d <partition> <image>      Dump partition to image file");
    println!("  blkops -h, --help                  Show this help message");
    println!();
    println!("Options:");
    println!("  -b, --block-size SIZE                Block size for dump/flash (default: 4M)");
    println!("  -c, --count BLOCKS                   Number of blocks to copy (for dump)");
    println!();
    println!("Examples:");
    println!("  blkops -s boot                    Find boot partition device path");
    println!("  blkops -f boot.img boot           Flash boot.img to boot partition");
    println!("  blkops -d boot boot.img           Dump boot partition to boot.img");
    println!("  blkops -d boot boot.img -b 1M -c 1024  Dump 1MB (1024 blocks) of boot partition");
    println!();
    println!("The tool automatically detects the current slot suffix (getprop ro.boot.slot_suffix)");
}
