use std::collections::HashMap;

use crate::errors;
use crate::{lang::AsmNode, symbol_table::SymbolTable};
use crate::lang::{AsmNodeValue, AsmSection, AsmInstruction};
use crate::tokenizer::SourceCode;

use hivmlib::{ByteCodes, VirtualAddress};


/// Generates byte code from the given assembly nodes.
/// Checks the types of the operands.
/// Resolves the still-unresolved symbols like $ or sections
pub fn generate(asm: Vec<AsmNode>, symbol_table: &SymbolTable, source: SourceCode) -> Vec<u8> {

    // Allocate a minumim starting capacity. 
    // The vector will most probably be reallocated, but this pre-allocation should avoid most minor initial reallocations.
    // In case all nodes are single-byte instructions, all reallocations are prevented.
    // In case the assembly code contains many non-code structures (sections, macros, etc.) this approach may avoid a reallocation.
    let mut bytecode = Vec::with_capacity(asm.len());

    let mut label_map: HashMap<&str, VirtualAddress> = HashMap::new();

    let mut current_section: Option<AsmSection> = None;

    for node in asm {

        match node.value {

            AsmNodeValue::Label(name) => {
                
                if current_section.is_none() {
                    errors::outside_section(&node.source, source, "Labels must be located inside an assembly section");
                }
                
                label_map.insert(name, VirtualAddress(bytecode.len()));
            },
            
            AsmNodeValue::Section(section) => {
                // Sections are secretly labels
                label_map.insert(section.name(), VirtualAddress(bytecode.len()));
                current_section = Some(section);
            },

            AsmNodeValue::Instruction(instruction) => {

                if current_section.is_none() {
                    errors::outside_section(&node.source, source, "Instructions must be located inside an assembly section");
                }

                match instruction {
                    AsmInstruction::AddInt1 => todo!(),
                    AsmInstruction::AddInt2 => todo!(),
                    AsmInstruction::AddInt4 => todo!(),
                    AsmInstruction::AddInt8 => todo!(),
                    AsmInstruction::SubInt1 => todo!(),
                    AsmInstruction::SubInt2 => todo!(),
                    AsmInstruction::SubInt4 => todo!(),
                    AsmInstruction::SubInt8 => todo!(),
                    AsmInstruction::MulInt1 => todo!(),
                    AsmInstruction::MulInt2 => todo!(),
                    AsmInstruction::MulInt4 => todo!(),
                    AsmInstruction::MulInt8 => todo!(),
                    AsmInstruction::DivInt1 => todo!(),
                    AsmInstruction::DivInt2 => todo!(),
                    AsmInstruction::DivInt4 => todo!(),
                    AsmInstruction::DivInt8 => todo!(),
                    AsmInstruction::ModInt1 => todo!(),
                    AsmInstruction::ModInt2 => todo!(),
                    AsmInstruction::ModInt4 => todo!(),
                    AsmInstruction::ModInt8 => todo!(),
                    AsmInstruction::AddFloat4 => todo!(),
                    AsmInstruction::AddFloat8 => todo!(),
                    AsmInstruction::SubFloat4 => todo!(),
                    AsmInstruction::SubFloat8 => todo!(),
                    AsmInstruction::MulFloat4 => todo!(),
                    AsmInstruction::MulFloat8 => todo!(),
                    AsmInstruction::DivFloat4 => todo!(),
                    AsmInstruction::DivFloat8 => todo!(),
                    AsmInstruction::ModFloat4 => todo!(),
                    AsmInstruction::ModFloat8 => todo!(),
                    AsmInstruction::LoadStatic1 { addr } => todo!(),
                    AsmInstruction::LoadStatic2 { addr } => todo!(),
                    AsmInstruction::LoadStatic4 { addr } => todo!(),
                    AsmInstruction::LoadStatic8 { addr } => todo!(),
                    AsmInstruction::LoadStaticBytes { addr } => todo!(),
                    AsmInstruction::LoadConst1 { value } => todo!(),
                    AsmInstruction::LoadConst2 { value } => todo!(),
                    AsmInstruction::LoadConst4 { value } => todo!(),
                    AsmInstruction::LoadConst8 { value } => todo!(),
                    AsmInstruction::LoadConstBytes { bytes } => todo!(),
                    AsmInstruction::Load1 => todo!(),
                    AsmInstruction::Load2 => todo!(),
                    AsmInstruction::Load4 => todo!(),
                    AsmInstruction::Load8 => todo!(),
                    AsmInstruction::LoadBytes => todo!(),
                    AsmInstruction::VirtualConstToReal { addr } => todo!(),
                    AsmInstruction::VirtualToReal => todo!(),
                    AsmInstruction::Store1 => todo!(),
                    AsmInstruction::Store2 => todo!(),
                    AsmInstruction::Store4 => todo!(),
                    AsmInstruction::Store8 => todo!(),
                    AsmInstruction::StoreBytes => todo!(),
                    AsmInstruction::Memmove1 => todo!(),
                    AsmInstruction::Memmove2 => todo!(),
                    AsmInstruction::Memmove4 => todo!(),
                    AsmInstruction::Memmove8 => todo!(),
                    AsmInstruction::MemmoveBytes => todo!(),
                    AsmInstruction::Duplicate1 => todo!(),
                    AsmInstruction::Duplicate2 => todo!(),
                    AsmInstruction::Duplicate4 => todo!(),
                    AsmInstruction::Duplicate8 => todo!(),
                    AsmInstruction::DuplicateBytes => todo!(),
                    AsmInstruction::Malloc => todo!(),
                    AsmInstruction::Realloc => todo!(),
                    AsmInstruction::Free => todo!(),
                    AsmInstruction::Intr => todo!(),
                    AsmInstruction::IntrConst { code } => todo!(),
                    AsmInstruction::Exit => todo!(),
                    AsmInstruction::JumpConst { addr } => todo!(),
                    AsmInstruction::Jump => todo!(),
                    AsmInstruction::JumpNotZeroConst1 { addr } => todo!(),
                    AsmInstruction::JumpNotZeroConst2 { addr } => todo!(),
                    AsmInstruction::JumpNotZeroConst4 { addr } => todo!(),
                    AsmInstruction::JumpNotZeroConst8 { addr } => todo!(),
                    AsmInstruction::JumpNotZero1 => todo!(),
                    AsmInstruction::JumpNotZero2 => todo!(),
                    AsmInstruction::JumpNotZero4 => todo!(),
                    AsmInstruction::JumpNotZero8 => todo!(),
                    AsmInstruction::JumpZeroConst1 { addr } => todo!(),
                    AsmInstruction::JumpZeroConst2 { addr } => todo!(),
                    AsmInstruction::JumpZeroConst4 { addr } => todo!(),
                    AsmInstruction::JumpZeroConst8 { addr } => todo!(),
                    AsmInstruction::JumpZero1 => todo!(),
                    AsmInstruction::JumpZero2 => todo!(),
                    AsmInstruction::JumpZero4 => todo!(),
                    AsmInstruction::JumpZero8 => todo!(),
                    AsmInstruction::Nop => todo!(),
                }
            }
        };

    }

    bytecode.shrink_to_fit();
    bytecode
}

