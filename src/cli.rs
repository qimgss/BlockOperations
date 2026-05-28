use clap::Parser;

#[derive(Parser)]
#[command(
    name = "blkops",
    author,
    version,
    about = "Android block device operation utility",
    long_about = "A pure Rust tool for finding, flashing, and dumping Android block devices"
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
    println!("blkops - Pure Rust Android Block Device Utility");
    println!();
    println!("Usage:");
    println!("  blkops -s, --search <partition>             Search for a partition and show its device path");
    println!("  blkops -f, --flash <image> <partition>      Flash image to partition (pure Rust)");
    println!("  blkops -d, --dump <partition> <image>       Dump partition to image file (pure Rust)");
    println!("  blkops -h, --help                           Show this help message");
    println!();
    println!("The tool automatically detects the current slot suffix (getprop ro.boot.slot_suffix)");
    println!("All operations are performed using pure Rust code without external dependencies.");
}