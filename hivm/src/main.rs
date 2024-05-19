mod exec;
mod cli_parser;

use std::fs;

use clap::Parser;
use cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    let bytecode = fs::read(args.input_file.as_path())
        .unwrap_or_else(|err| panic!("Could not read input file \"{}\".\n{err}", args.input_file.display()));

    let mut vm = exec::VM::new(args.stack_size);

    let code = vm.run(&bytecode);

    println!("Process exited with code {code}");
    std::process::exit(code as i32);
}

