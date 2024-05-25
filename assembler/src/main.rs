#![feature(variant_count)]

mod cli_parser;
mod files;
mod assembler;
mod errors;
mod tokenizer;
mod symbol_table;
mod parser;
mod lang;
mod code_generator;

use clap::Parser;
use cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    let bytecode = assembler::assemble(&args.input_file);

    if let Some(err) = files::save_byte_code(&bytecode.into_boxed_slice(), &args.input_file).err() {
        errors::io_error(err, "Could not save byte code file.");
    }

}

