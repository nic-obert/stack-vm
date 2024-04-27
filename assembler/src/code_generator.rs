use std::collections::HashMap;

use crate::symbol_table::SymbolTable;
use crate::lang::AsmNode;
use crate::tokenizer::SourceCode;

use hivmlib::{ByteCodes, VirtualAddress};


/// Generates byte code from the given assembly nodes.
/// Checks the types of the operands.
/// Resolves the still-unresolved symbols like $ or sections
pub fn generate(asm: &[AsmNode], symbol_table: &SymbolTable, source: SourceCode) -> Vec<u8> {

    // Allocate a minumim starting capacity. 
    // The vector will most probably be reallocated, but this pre-allocation should avoid most minor initial reallocations.
    // In case all nodes are single-byte instructions, all reallocations are prevented.
    // In case the assembly code contains many non-code structures (sections, macros, etc.) this approach may avoid a reallocation.
    let mut bytecode = Vec::with_capacity(asm.len());

    let mut label_map: HashMap<&str, VirtualAddress> = HashMap::new();

    bytecode.shrink_to_fit();
    bytecode
}

