use std::io;

use crate::assembler::SourceToken;


pub fn io_error(err: io::Error) -> ! {
    eprintln!("IO error: {}", err);
    std::process::exit(1);
}


pub fn parsing_error(token: &SourceToken, message: &str) -> ! {
    eprintln!("Parsing error at {}:{}:\n{}", token.line_number(), token.column, message);
    std::process::exit(1);
}

