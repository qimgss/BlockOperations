mod cli;
mod flash;
mod device;
mod utils;

use anyhow::Result;
use cli::Cli;
use device::BlockDeviceFinder;
use flash::{ImageFlasher, ImageDumper};

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.should_show_help() {
        cli::print_help();
        return Ok(());
    }
    
    if let Some(partition_name) = &cli.search {
        handle_search(partition_name)?;
    }
    
    if let Some(args) = &cli.flash {
        if args.len() >= 2 {
            handle_flash(&args[0], &args[1])?;
        } else {
            println!("Error: flash command requires image and partition arguments");
        }
    }
    
    if let Some(args) = &cli.dump {
        if args.len() >= 2 {
            let block_size = cli.block_size.as_deref().unwrap_or("4M");
            handle_dump(&args[0], &args[1], block_size, cli.count.as_deref())?;
        } else {
            println!("Error: dump command requires partition and image arguments");
        }
    }
    
    Ok(())
}

fn handle_search(partition_name: &str) -> Result<()> {
    println!("Searching for partition: {}", partition_name);
    let finder = BlockDeviceFinder::new();
    let slot_suffix = finder.get_slot_suffix()?;
    println!("Detected slot suffix: {}", if slot_suffix.is_empty() { "none" } else { &slot_suffix });
    
    let device_path = finder.find_partition(partition_name, &slot_suffix)?;
    println!("Found: {}", device_path);
    Ok(())
}

fn handle_flash(image_path: &str, partition_name: &str) -> Result<()> {
    println!("Flashing {} to partition: {}", image_path, partition_name);
    let finder = BlockDeviceFinder::new();
    let slot_suffix = finder.get_slot_suffix()?;
    println!("Detected slot suffix: {}", if slot_suffix.is_empty() { "none" } else { &slot_suffix });
    
    let target_device = finder.find_partition(partition_name, &slot_suffix)?;
    println!("Target device: {}", target_device);
    
    let flasher = ImageFlasher::new();
    flasher.flash_image(image_path, &target_device)?;
    println!("Successfully flashed {} to {}", image_path, target_device);
    Ok(())
}

fn handle_dump(partition_name: &str, output_path: &str, block_size: &str, count: Option<&str>) -> Result<()> {
    println!("Dumping partition {} to {}", partition_name, output_path);
    let finder = BlockDeviceFinder::new();
    let slot_suffix = finder.get_slot_suffix()?;
    println!("Detected slot suffix: {}", if slot_suffix.is_empty() { "none" } else { &slot_suffix });
    
    let source_device = finder.find_partition(partition_name, &slot_suffix)?;
    println!("Source device: {}", source_device);
    
    let dumper = ImageDumper::new();
    dumper.dump_partition(&source_device, output_path, block_size, count)?;
    println!("Successfully dumped {} to {}", partition_name, output_path);
    Ok(())
}
