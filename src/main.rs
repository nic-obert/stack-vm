mod exec;
mod bytecode;
mod cli_parser;

use clap::Parser;
use cli_parser::CliParser;

fn main() {
    
    let args = CliParser::parse();

}

