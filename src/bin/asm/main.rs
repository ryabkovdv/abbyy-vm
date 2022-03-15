mod ast;
mod compiler;
mod id_table;
mod inst_syms;
mod lexer;
mod parser;

use std::fs;
use std::path::Path;

use my_vm::binfile;

use crate::compiler::compile;
use crate::inst_syms::make_proper_id_table;
use crate::lexer::Lexer;
use crate::parser::parse;

struct Error;

fn run() -> Result<(), Error> {
    let args: Vec<_> = std::env::args_os().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} SOURCE OUTPUT.", Path::new(&args[0]).display());
        return Err(Error);
    }

    let source_name = Path::new(&args[1]);
    let output_name = Path::new(&args[2]);

    let source = match fs::read_to_string(source_name) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("Failed to load file {}: {}.", source_name.display(), err);
            return Err(Error);
        }
    };

    let mut lexer = Lexer::new(&source);
    let mut id_table = make_proper_id_table();
    let mut ast = Vec::new();
    match parse(&mut lexer, &mut id_table, &mut ast) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Error in line {}: {}.", err.line, err);
            return Err(Error);
        }
    }

    let program = match compile(&ast, &id_table) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("Error in line {}: {}.", err.line, err);
            return Err(Error);
        }
    };

    let mut output = Vec::new();
    match binfile::serialize(program.memory_size, &program.segments, &mut output) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Failed to serialize file {}: {}.", output_name.display(), err);
            return Err(Error);
        }
    }

    match fs::write(output_name, &output) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Failed to write file {}: {}.", output_name.display(), err);
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
