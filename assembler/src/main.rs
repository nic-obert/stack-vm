#![feature(variant_count)]
#![feature(cell_leak)]
#![feature(io_error_more)]
#![feature(os_str_display)]

mod cli_parser;
mod files;
mod assembler;
mod errors;
mod tokenizer;
mod symbol_table;
mod parser;
mod lang;
mod code_generator;
mod module_manager;

use std::env;

use clap::Parser;
use cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    let cwd = env::current_dir()
        .unwrap_or_else( |err| errors::io_error(err, "Failed to resolve current directory path."));

    let bytecode = assembler::assemble(&cwd, &args.input_file, args.include_paths);

    if let Some(err) = files::save_byte_code(&bytecode.into_boxed_slice(), &args.input_file).err() {
        errors::io_error(err, "Could not save byte code file.");
    }

}

