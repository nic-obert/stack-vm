use std::path::Path;
use std::path::PathBuf;

use crate::errors;
use crate::files;
use crate::lang::AsmNode;
use crate::module_manager::AsmUnit;
use crate::module_manager::ModuleManager;
use crate::parser::MacroMap;
use crate::symbol_table::SymbolTable;
use crate::tokenizer;
use crate::parser;
use crate::code_generator;


pub fn load_unit_asm<'a>(caller_directory: Option<&Path>, unit_path: &'a Path, symbol_table: &'a SymbolTable<'a>, module_manager: &'a ModuleManager<'a>, macros: &mut MacroMap<'a>) -> Vec<AsmNode<'a>> {

    // Shadow the previous `unit_path` to avoid confusion with the variables
    let unit_path = module_manager.resolve_include_path(caller_directory, unit_path)
        .unwrap_or_else(|err| 
            errors::io_error(err, format!("Failed to canonicalize path \"{}\"", unit_path.display()).as_str()));

    if module_manager.is_loaded(&unit_path) {
        // If the module was already imported, return an empty assembly
        return Vec::new();
    }

    let raw_source = files::load_assembly(unit_path)
        .unwrap_or_else(|err| errors::io_error(err, format!("Could not load file \"{}\"", unit_path.display()).as_str()));

    let asm_unit = module_manager.add_unit(unit_path, AsmUnit::new(raw_source));

    let token_lines = tokenizer::tokenize(&asm_unit.lines, unit_path, symbol_table, module_manager);

    println!("\nTokens:\n");
    for line in &token_lines {
        for token in line.iter() {
            println!("{}", token);
        }
    }

    let asm = parser::parse(token_lines, symbol_table, module_manager, macros);

    println!("\n\nNodes:\n");
    for node in &asm {
        println!("{:?}", node);
    }

    asm
}


pub fn assemble(caller_directory: &Path, unit_path: &Path, include_paths: Vec<PathBuf>) -> Vec<u8> {

    let symbol_table = SymbolTable::new();
    let mut macros = MacroMap::new();
    
    let module_manager = ModuleManager::new(include_paths);
    
    let asm = load_unit_asm(Some(caller_directory), unit_path, &symbol_table, &module_manager, &mut macros);

    code_generator::generate(&asm, &symbol_table, &module_manager)
}

