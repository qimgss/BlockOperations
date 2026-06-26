use clap::Parser;

#[derive(Parser)]
#[command(
    name = "blkops",
    author,
    version,
    about = "Pure Rust Android block device operation utility",
    long_about = "A pure Rust tool for finding, flashing, and dumping Android block devices"
)]
pub struct Cli {
    /// Search for a partition and show its device path
    #[arg(short = 's', long = "search", value_name = "PARTITION")]
    pub search: Option<String>,
    
    /// Only show the device path when searching (no extra info)
    #[arg(short = 'p', long = "path", requires = "search")]
    pub path_only: bool,
    
    /// Flash an image to a partition (alias for --write)
    #[arg(short = 'f', long = "flash", value_name = "IMAGE", num_args = 2, value_names = &["IMAGE", "PARTITION"], hide = true)]
    pub flash: Option<Vec<String>>,
    
    /// Write an image to a partition
    #[arg(short = 'w', long = "write", value_name = "IMAGE", num_args = 2, value_names = &["IMAGE", "PARTITION"])]
    pub write: Option<Vec<String>>,
    
    /// Dump (extract) a partition to an image file
    #[arg(short = 'd', long = "dump", value_name = "PARTITION", num_args = 2, value_names = &["PARTITION", "IMAGE"])]
    pub dump: Option<Vec<String>>,
    
    /// Show help information
    #[arg(short = 'h', long = "help")]
    pub help: bool,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
    
    pub fn should_show_help(&self) -> bool {
        self.help || (self.search.is_none() && self.flash.is_none() && self.write.is_none() && self.dump.is_none())
    }
}

pub fn print_help() {
    println!("blkops - Pure Rust Android Block Device Utility");
    println!();
    println!("Usage:");
    println!("  blkops -s <partition>              Search for a partition and show its device path");
    println!("  blkops -s <partition> -p           Search and show only the device path");
    println!("  blkops -w <image> <partition>      Write image to partition");
    println!("  blkops -d <partition> <image>      Dump partition to image file");
    println!("  blkops -h, --help                  Show this help message");
    println!();
    println!("The tool automatically detects the current slot suffix (getprop ro.boot.slot_suffix)");
    println!("All operations are performed using pure Rust code without external dependencies.");
}