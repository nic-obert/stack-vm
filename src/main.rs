mod exec;
mod bytecode;
mod cli_parser;

use std::fs;

use clap::Parser;
use cli_parser::CliParser;



fn main() {
    
    let args = CliParser::parse();

    let bytecode = fs::read(args.input_file.as_path())
        .unwrap_or_else(|err| panic!("Could not read input file \"{}\".\n{err}", args.input_file.display()));

    let mut vm = exec::VM::new(args.stack_size);

    vm.run(&bytecode);

}

