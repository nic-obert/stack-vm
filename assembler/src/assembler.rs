use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::path::Path;

use crate::errors;
use crate::files;
use crate::lang::AsmNode;
use crate::symbol_table::SymbolTable;
use crate::tokenizer;
use crate::parser;
use crate::code_generator;


/// Struct must not implement Clone or Copy
pub struct AsmUnit {
    /// The actual "owned" memory address of the source code string
    raw_source: &'static str,
    pub lines: Box<[&'static str]>,
}

impl AsmUnit {

    pub fn new(raw_source: String) -> Self {
        
        let raw_source = Box::leak(raw_source.into_boxed_str());

        let mut lines = Vec::new();
        for line in raw_source.lines() {
            lines.push( unsafe {
                std::str::from_utf8_unchecked(line.as_bytes())
            });
        }

        Self {
            lines: lines.into_boxed_slice(),
            raw_source,
        }
    }

}

impl Drop for AsmUnit {

    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.raw_source as *const str as *mut str));
        }
    }

}


pub struct ModuleManager<'a> {

    // Here a Box is used to allow mutating the `units` HashMap without invalidating references to the AsmUnit
    // The Box itself will change, but not the address it points to
    units: UnsafeCell<HashMap<&'a Path, Box<AsmUnit>>>,

}

impl<'a> ModuleManager<'a> {

    pub fn new() -> Self {
        Self {
            units: Default::default()
        }
    }


    pub fn add_unit(&self, path: &'a Path, unit: AsmUnit) -> &'a AsmUnit {

        // This is safe because no references to the map or its elements is ever returned
        let units = unsafe { &mut *self.units.get() };

        let unit_box = Box::new(unit);
        let unit_ref = unit_box.as_ref() as *const AsmUnit;

        units.insert(path, unit_box);
        // Returns a ref to the newly added unit. Since the unit is stored in the heap and is never mutated, its memory address won't change
        // and the reference will be valid for the lifetime of the module manager
        unsafe {
            &*unit_ref as &AsmUnit
        }
    }


    /// Get an immutable reference to the assembly unit
    pub fn get_unit(&self, path: &Path) -> &'a AsmUnit {
        let units = unsafe { &*self.units.get() };
        units.get(path).expect("Entry should exist")
    }


    pub fn is_loaded(&self, path: &Path) -> bool {
        let units = unsafe { &*self.units.get() };
        units.contains_key(path)
    }

}


pub fn load_unit_asm<'a>(unit_path: &'a Path, symbol_table: &'a SymbolTable<'a>, module_manager: &'a ModuleManager<'a>) -> Vec<AsmNode<'a>> {

    if module_manager.is_loaded(unit_path) {
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

    let asm = parser::parse(token_lines, symbol_table, module_manager);

    println!("\n\nNodes:\n");
    for node in &asm {
        println!("{:?}", node);
    }

    asm
}


pub fn assemble(unit_path: &Path) -> Vec<u8> {

    let symbol_table = SymbolTable::new();
    
    let module_manager = ModuleManager::new();
    
    let asm = load_unit_asm(unit_path, &symbol_table, &module_manager);

    code_generator::generate(&asm, &symbol_table, &module_manager)
}

