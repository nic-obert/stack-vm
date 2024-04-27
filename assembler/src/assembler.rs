use std::path::Path;

use crate::symbol_table::SymbolTable;
use crate::tokenizer;
use crate::parser;
use crate::code_generator;


pub fn assemble<'a>(raw_source: &'a str, unit_path: &'a Path) -> Vec<u8> {

    let source_lines = raw_source.lines().collect::<Vec<_>>();
    
    let mut symbol_table = SymbolTable::new();

    let token_lines = tokenizer::tokenize(&source_lines, unit_path, &mut symbol_table);

    println!("\nTokens:\n");
    for line in &token_lines {
        for token in line.iter() {
            println!("{}", token);
        }
    }

    let asm = parser::parse(&token_lines, &source_lines, &symbol_table);

    println!("\n\nNodes:\n");
    for node in &asm {
        println!("{:?}", node);
    }

    code_generator::generate(&asm, &symbol_table, &source_lines)
}

