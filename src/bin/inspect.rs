use std::fs;
use std::path::Path;

use my_vm::binfile;

struct Error;

fn run() -> Result<(), Error> {
    let args: Vec<_> = std::env::args_os().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} FILE.", Path::new(&args[0]).display());
        return Err(Error);
    }

    let file_name = Path::new(&args[1]);
    let file_data = match fs::read(file_name) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to load file {}: {}.", file_name.display(), err);
            return Err(Error);
        }
    };

    let file = match binfile::File::from_bytes(&file_data) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to load file {}: {}.", file_name.display(), err);
            return Err(Error);
        }
    };

    println!("File size:     0x{:X}", file_data.len());
    println!("Memory size:   0x{:X}", file.memory_size());
    println!("Segment count: {}", file.segment_count());

    for (i, segment) in file.raw_segments().enumerate() {
        println!("Segment {}:", i);
        println!("\tOffset:  0x{:X}", segment.offset);
        println!("\tAddress: 0x{:X}", segment.addr);
        println!("\tSize:    0x{:X}", segment.size);
    }

    Ok(())
}

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
