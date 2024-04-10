use std::io;
use std::cmp::min;

use colored::Colorize;

use crate::assembler::{SourceToken, SourceCode};


pub fn print_source_context(source: SourceCode, line_index: usize, char_pointer: usize) {

    /// Number of lines of source code to include before and after the highlighted line in error messages
    const SOURCE_CONTEXT_RADIUS: u8 = 3;

    // Calculate the beginning of the context. Saturating subtraction is used interpret underflow as 0.
    let mut index = line_index.saturating_sub(SOURCE_CONTEXT_RADIUS as usize);
    let end_index = min(line_index + SOURCE_CONTEXT_RADIUS as usize + 1, source.len());

    let line_number_width = end_index.to_string().len();
    
    // Print the source lines before the highlighted line.
    while index < line_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }

    // The highlighted line.
    println!("{}{:line_number_width$}  {}", ">".bright_red().bold(), index + 1, source[line_index]);
    println!(" {:line_number_width$} {:>char_pointer$}{}", "", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }
}



pub fn io_error(err: io::Error) -> ! {
    eprintln!("IO error: {}", err);
    std::process::exit(1);
}


pub fn invalid_escape_sequence(token: &SourceToken, character: char, char_index: usize, source: SourceCode) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Invalid escape sequence at {}:{}: `{character}`", token.line_number(), char_index);

    print_source_context(source, token.line_index, char_index);

    std::process::exit(1);
}


pub fn parsing_error(token: &SourceToken, source: SourceCode, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Parsing error at {}:{} on token `{}`:\n{}", token.line_number(), token.column, token.string, message);

    print_source_context(source, token.line_index, token.column);

    std::process::exit(1);
}

