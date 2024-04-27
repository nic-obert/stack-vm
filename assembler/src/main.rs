#![feature(variant_count)]

mod cli_parser;
mod files;
mod assembler;
mod errors;
mod tokenizer;
mod symbol_table;
mod parser;
mod lang;

use clap::Parser;
use cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    let assembly = files::load_assembly(&args.input_file)
        .unwrap_or_else(|err| errors::io_error(err));

    let bytecode = assembler::assemble(&assembly, &args.input_file);

    if let Some(err) = files::save_byte_code(&bytecode.into_boxed_slice(), &args.input_file).err() {
        errors::io_error(err);
    }

}

