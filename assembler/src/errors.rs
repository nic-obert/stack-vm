use std::io;
use std::cmp::min;

use colored::Colorize;

use crate::tokenizer::{SourceCode, SourceToken};
use crate::symbol_table::Symbol;
use crate::module_manager::ModuleManager;


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


pub fn io_error(err: io::Error, message: &str) -> ! {
    eprintln!("IO error: {err}\n{message}");
    std::process::exit(1);
}


pub fn invalid_escape_sequence(token: &SourceToken, character: char, char_index: usize, source: SourceCode) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Invalid escape sequence at {}:{}: `{character}`", token.line_number(), char_index);

    print_source_context(source, token.line_index, char_index);

    std::process::exit(1);
}


pub fn symbol_redeclaration<'a>(token: &SourceToken, module_manager: &'a ModuleManager<'a>, old_declaration: &Symbol) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Symbol redeclaration at {}:{}: `{}`", token.line_number(), token.column, old_declaration.name);
    
    print_source_context(&module_manager.get_unit(token.unit_path).lines, token.line_index, token.column);
    
    eprintln!("\nPrevious declaration at {}:{}: `{}`", old_declaration.source.line_number(), old_declaration.source.column, old_declaration.name);

    print_source_context(&module_manager.get_unit(old_declaration.source.unit_path).lines, old_declaration.source.line_index, old_declaration.source.column);
    
    std::process::exit(1);

}


pub fn parsing_error<'a>(token: &SourceToken, module_manager: &'a ModuleManager<'a>, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Parsing error at {}:{} on token `{}`:\n{}", token.line_number(), token.column, token.string, message);

    print_source_context(&module_manager.get_unit(token.unit_path).lines, token.line_index, token.column);

    std::process::exit(1);
}


pub fn tokenizer_error(token: &SourceToken, source: SourceCode, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Tokenizer error at {}:{} on token `{}`:\n{}", token.line_number(), token.column, token.string, message);

    print_source_context(source, token.line_index, token.column);

    std::process::exit(1);
}


pub fn outside_section<'a>(token: &SourceToken, module_manager: &'a ModuleManager<'a>, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Item outside an assembly section section at {}:{}: {}", token.line_number(), token.column, message);

    print_source_context(&module_manager.get_unit(token.unit_path).lines, token.line_index, token.column);

    std::process::exit(1);
}


pub fn invalid_argument<'a>(token: &SourceToken, module_manager: &'a ModuleManager<'a>, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Invalid argument at {}:{}: {}", token.line_number(), token.column, message);

    print_source_context(&module_manager.get_unit(token.unit_path).lines, token.line_index, token.column);

    std::process::exit(1);
}


pub fn undefined_symbol<'a>(token: &SourceToken, module_manager: &'a ModuleManager<'a>) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path.display());
    eprintln!("Undefined symbol at {}:{}: `{}`", token.line_number(), token.column, token.string);

    print_source_context(&module_manager.get_unit(token.unit_path).lines, token.line_index, token.column);

    std::process::exit(1);
}
