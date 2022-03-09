use std::fs;
use std::io::{stdin, stdout, BufReader, LineWriter, Read, Stdin, Stdout, Write};
use std::path::Path;

use my_vm::{binfile, vm};

struct Sysfn {
    stdin: BufReader<Stdin>,
    stdout: LineWriter<Stdout>,
}

impl vm::Sysfn for Sysfn {
    fn read(&mut self) -> u32 {
        self.stdout.flush().unwrap();

        let mut buf = [0_u8];
        match self.stdin.read_exact(&mut buf) {
            Ok(()) => buf[0] as u32,
            Err(_) => !0,
        }
    }

    fn write(&mut self, value: u32) {
        let buf = [value as u8];
        self.stdout.write_all(&buf).unwrap();
    }
}

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

    let mut memory = match binfile::to_memory(&file_data) {
        Ok(memory) => memory,
        Err(err) => {
            eprintln!("Failed to load file {}: {}.", file_name.display(), err);
            return Err(Error);
        }
    };

    let mut sysfn = Sysfn {
        stdin: BufReader::new(stdin()),
        stdout: LineWriter::new(stdout()),
    };

    let mut state = vm::State::new();
    match vm::run(&mut state, &mut memory, &mut sysfn) {
        Ok(status) => {
            eprintln!("Success (exit status {}).", status);
        }
        Err(err) => {
            eprintln!("Error: {}.", err);
            eprint!("State:\n{}", state);
            return Err(Error);
        }
    }

    Ok(())
}

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
