use std::collections::HashMap;
use std::rc::Rc;

use crate::errors;
use crate::{lang::AsmNode, symbol_table::SymbolTable};
use crate::lang::{AsmNodeValue, AsmSection, AsmInstruction, AddressLike, NumberLike, Number};
use crate::tokenizer::{SourceCode, SourceToken};

use hivmlib::{ByteCodes, VirtualAddress, ADDRESS_SIZE, INSTRUCTION_SIZE};


struct UnresolvedLabel<'a> {
    location: VirtualAddress,
    name: &'a str,
    source: Rc<SourceToken<'a>>
}


/// Generates byte code from the given assembly nodes.
/// Checks the types of the operands.
/// Resolves the still-unresolved symbols like $ or sections
/// Some symbols must be resolved at this stage like $, sections, and labels because they depend on the generated code.
pub fn generate(asm: Vec<AsmNode>, symbol_table: &SymbolTable, source: SourceCode) -> Vec<u8> {

    // Allocate a minumim starting capacity. 
    // The vector will most probably be reallocated, but this pre-allocation should avoid most minor initial reallocations.
    // In case all nodes are single-byte instructions, all reallocations are prevented.
    // In case the assembly code contains many non-code structures (sections, macros, etc.) this approach may avoid a reallocation.
    // + 9 to include an initial jump instruction to the entry point (1-byte instruction + 8-byte address)
    let mut bytecode = Vec::with_capacity(asm.len() + ADDRESS_SIZE + INSTRUCTION_SIZE);

    let mut label_map: HashMap<&str, VirtualAddress> = HashMap::new();
    let mut unresolved_labels: Vec<UnresolvedLabel> = Vec::new();

    let mut current_section: Option<AsmSection> = None;

    bytecode.push(ByteCodes::JumpConst as u8);
    bytecode.extend_from_slice([0u8; 8].as_ref()); // A placeholder for the entry point address, will be filled later

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


                macro_rules! one_arg_address_instruction {
                    ($name:ident, $addr:ident) => {{
                        let operand: VirtualAddress = match $addr.0 {

                            AddressLike::Number(n) => {
                                if let Some(n) = n.as_uint() {
                                    VirtualAddress(n as usize)
                                } else {
                                    errors::invalid_argument(&$addr.1, source, format!("Invalid address `{:?}`. Must be a positive integer.", n).as_str())
                                }
                            },

                            AddressLike::Symbol(id) => VirtualAddress(
                                get_symbol_or_placeholder!(id, &$addr.1, |value, symbol| 
                                    value.as_uint(symbol_table)
                                    .unwrap_or_else(
                                        || errors::invalid_argument(&$addr.1, source, format!("Invalid address `{:?}`. Must be a positive integer.", symbol.value).as_str())
                                    )
                                ) as usize),

                            AddressLike::CurrentPosition => VirtualAddress(bytecode.len()),
                        };

                        bytecode.push(ByteCodes::$name as u8);
                        bytecode.extend_from_slice(&operand.to_le_bytes());
                    }}
                }

                macro_rules! one_arg_number_instruction {
                    ($name:ident, $value:ident, $size:literal) => {{

                        let (bytes, minimum_size) = match &$value.0 {

                            NumberLike::Number(n, size) => (n.as_le_bytes(), *size as usize),

                            NumberLike::Symbol(id) => (
                                get_symbol_or_placeholder!(*id, &$value.1, |value, symbol| 
                                    value.as_uint(symbol_table)
                                    .unwrap_or_else(
                                        || errors::invalid_argument(&$value.1, source, format!("Invalid address `{:?}`. Must be a positive integer.", symbol.value).as_str())
                                    )
                                ).to_le_bytes().to_vec(),
                                ADDRESS_SIZE
                            ),
                            
                            NumberLike::CurrentPosition => (bytecode.len().to_le_bytes().to_vec(), ADDRESS_SIZE),
                        };

                        if minimum_size > $size {
                            errors::invalid_argument(&$value.1, source, format!("Invalid constant value `{:?}`. Must be a {} bytes integer, but got length of {} bytes.", $value.0, $size, minimum_size).as_str());
                        }

                        bytecode.push(ByteCodes::$name as u8);
                        bytecode.extend_from_slice(&bytes[0..$size]);
                    }}
                }

                macro_rules! get_symbol_or_placeholder {
                    ($symbol_id:expr, $arg_source:expr, |$value:ident, $symbol:ident| $operations:stmt) => {{

                        let $symbol = symbol_table.get_symbol($symbol_id).borrow();

                        if let Some($value) = $symbol.value.as_ref() {
                            $operations
                        } else {
                            // The symbol is not defined yet, so we need to resolve it later
                            unresolved_labels.push(UnresolvedLabel {
                                location: VirtualAddress(bytecode.len()),
                                name: $symbol.name,
                                source: Rc::clone($arg_source)
                            });

                            // Return a placeholder value that will be replaced later during the final symbol resolution
                            0
                        }
                    }};
                }


                match instruction {
                    AsmInstruction::AddInt1 => bytecode.push(ByteCodes::AddInt1 as u8),
                    AsmInstruction::AddInt2 => bytecode.push(ByteCodes::AddInt2 as u8),
                    AsmInstruction::AddInt4 => bytecode.push(ByteCodes::AddInt4 as u8),
                    AsmInstruction::AddInt8 => bytecode.push(ByteCodes::AddInt8 as u8),
                    AsmInstruction::SubInt1 => bytecode.push(ByteCodes::SubInt1 as u8),
                    AsmInstruction::SubInt2 => bytecode.push(ByteCodes::SubInt2 as u8),
                    AsmInstruction::SubInt4 => bytecode.push(ByteCodes::SubInt4 as u8),
                    AsmInstruction::SubInt8 => bytecode.push(ByteCodes::SubInt8 as u8),
                    AsmInstruction::MulInt1 => bytecode.push(ByteCodes::MulInt1 as u8),
                    AsmInstruction::MulInt2 => bytecode.push(ByteCodes::MulInt2 as u8),
                    AsmInstruction::MulInt4 => bytecode.push(ByteCodes::MulInt4 as u8),
                    AsmInstruction::MulInt8 => bytecode.push(ByteCodes::MulInt8 as u8),
                    AsmInstruction::DivInt1 => bytecode.push(ByteCodes::DivInt1 as u8),
                    AsmInstruction::DivInt2 => bytecode.push(ByteCodes::DivInt2 as u8),
                    AsmInstruction::DivInt4 => bytecode.push(ByteCodes::DivInt4 as u8),
                    AsmInstruction::DivInt8 => bytecode.push(ByteCodes::DivInt8 as u8),
                    AsmInstruction::ModInt1 => bytecode.push(ByteCodes::ModInt1 as u8),
                    AsmInstruction::ModInt2 => bytecode.push(ByteCodes::ModInt2 as u8),
                    AsmInstruction::ModInt4 => bytecode.push(ByteCodes::ModInt4 as u8),
                    AsmInstruction::ModInt8 => bytecode.push(ByteCodes::ModInt8 as u8),
                    AsmInstruction::AddFloat4 => bytecode.push(ByteCodes::AddFloat4 as u8),
                    AsmInstruction::AddFloat8 => bytecode.push(ByteCodes::AddFloat8 as u8),
                    AsmInstruction::SubFloat4 => bytecode.push(ByteCodes::SubFloat4 as u8),
                    AsmInstruction::SubFloat8 => bytecode.push(ByteCodes::SubFloat8 as u8),
                    AsmInstruction::MulFloat4 => bytecode.push(ByteCodes::MulFloat4 as u8),
                    AsmInstruction::MulFloat8 => bytecode.push(ByteCodes::MulFloat8 as u8),
                    AsmInstruction::DivFloat4 => bytecode.push(ByteCodes::DivFloat4 as u8),
                    AsmInstruction::DivFloat8 => bytecode.push(ByteCodes::DivFloat8 as u8),
                    AsmInstruction::ModFloat4 => bytecode.push(ByteCodes::ModFloat4 as u8),
                    AsmInstruction::ModFloat8 => bytecode.push(ByteCodes::ModFloat8 as u8),
                    AsmInstruction::LoadStatic1 { addr } => one_arg_address_instruction!(LoadStatic1, addr),
                    AsmInstruction::LoadStatic2 { addr } => one_arg_address_instruction!(LoadStatic2, addr),
                    AsmInstruction::LoadStatic4 { addr } => one_arg_address_instruction!(LoadStatic4, addr),
                    AsmInstruction::LoadStatic8 { addr } => one_arg_address_instruction!(LoadStatic8, addr),

                    AsmInstruction::LoadStaticBytes { addr, count } => {

                        one_arg_address_instruction!(LoadStaticBytes, addr);
                        
                        let count = match count.0 {
                                
                                NumberLike::Number(n, _size) => {
                                    if let Some(n) = n.as_uint() {
                                        n as usize
                                    } else {
                                        errors::invalid_argument(&count.1, source, format!("Invalid count `{:?}`. Must be a positive integer.", n).as_str())
                                    }
                                },
    
                                NumberLike::Symbol(id)
                                 => get_symbol_or_placeholder!(id, &count.1, |value, symbol| 
                                        value.as_uint(symbol_table)
                                        .unwrap_or_else(
                                            || errors::invalid_argument(&count.1, source, format!("Invalid count `{:?}`. Must be a positive integer.", symbol.value).as_str())
                                        ) as usize
                                    ),
    
                                NumberLike::CurrentPosition => bytecode.len(),
                            };

                        bytecode.extend(count.to_le_bytes());
                    },

                    AsmInstruction::LoadConst1 { value } => one_arg_number_instruction!(LoadConst1, value, 1),
                    AsmInstruction::LoadConst2 { value } => one_arg_number_instruction!(LoadConst2, value, 2),
                    AsmInstruction::LoadConst4 { value } => one_arg_number_instruction!(LoadConst4, value, 4),
                    AsmInstruction::LoadConst8 { value } => one_arg_number_instruction!(LoadConst8, value, 8),

                    AsmInstruction::LoadConstBytes { bytes } => {

                        let mut value_bytes = Vec::with_capacity(bytes.len());
                        
                        for byte in bytes {

                            let bytes_per_byte = match &byte.0 {
                                
                                NumberLike::Number(n, _size) => {
                                    if matches!(n, Number::Float(_)) {
                                        errors::invalid_argument(&byte.1, source, format!("Invalid constant value `{:?}`. Must be an integer, not a float.", n).as_str())
                                    }

                                    n.as_le_bytes()
                                },

                                NumberLike::Symbol(id) 
                                 => get_symbol_or_placeholder!(*id, &byte.1, |value, symbol|
                                        value.as_uint(symbol_table)
                                        .unwrap_or_else(
                                            || errors::invalid_argument(&byte.1, source, format!("Invalid address `{:?}`. Must be a positive integer.", symbol.value).as_str())
                                    ))
                                    .to_le_bytes().to_vec(),

                                NumberLike::CurrentPosition => bytecode.len().to_le_bytes().to_vec(),
                            };

                            if bytes_per_byte.len() != 1 {
                                errors::invalid_argument(&byte.1, source, format!("Invalid constant value `{:?}`. Must be a single byte but {} bytes were provided.", byte.0, bytes_per_byte.len()).as_str())
                            }

                            value_bytes.push(bytes_per_byte[0]);
                        }

                        bytecode.push(ByteCodes::LoadConstBytes as u8);
                        bytecode.extend(value_bytes.len().to_le_bytes());
                        bytecode.extend_from_slice(value_bytes.as_slice());
                    },

                    AsmInstruction::Load1 => bytecode.push(ByteCodes::Load1 as u8),
                    AsmInstruction::Load2 => bytecode.push(ByteCodes::Load2 as u8),
                    AsmInstruction::Load4 => bytecode.push(ByteCodes::Load4 as u8),
                    AsmInstruction::Load8 => bytecode.push(ByteCodes::Load8 as u8),
                    AsmInstruction::LoadBytes => bytecode.push(ByteCodes::LoadBytes as u8),
                    AsmInstruction::VirtualConstToReal { addr } => one_arg_address_instruction!(VirtualConstToReal, addr),
                    AsmInstruction::VirtualToReal => bytecode.push(ByteCodes::VirtualToReal as u8),
                    AsmInstruction::Store1 => bytecode.push(ByteCodes::Store1 as u8),
                    AsmInstruction::Store2 => bytecode.push(ByteCodes::Store2 as u8),
                    AsmInstruction::Store4 => bytecode.push(ByteCodes::Store4 as u8),
                    AsmInstruction::Store8 => bytecode.push(ByteCodes::Store8 as u8),
                    AsmInstruction::StoreBytes => bytecode.push(ByteCodes::StoreBytes as u8),
                    AsmInstruction::Memmove1 => bytecode.push(ByteCodes::Memmove1 as u8),
                    AsmInstruction::Memmove2 => bytecode.push(ByteCodes::Memmove2 as u8),
                    AsmInstruction::Memmove4 => bytecode.push(ByteCodes::Memmove4 as u8),
                    AsmInstruction::Memmove8 => bytecode.push(ByteCodes::Memmove8 as u8),
                    AsmInstruction::MemmoveBytes => bytecode.push(ByteCodes::MemmoveBytes as u8),
                    AsmInstruction::Duplicate1 => bytecode.push(ByteCodes::Duplicate1 as u8),
                    AsmInstruction::Duplicate2 => bytecode.push(ByteCodes::Duplicate2 as u8),
                    AsmInstruction::Duplicate4 => bytecode.push(ByteCodes::Duplicate4 as u8),
                    AsmInstruction::Duplicate8 => bytecode.push(ByteCodes::Duplicate8 as u8),
                    AsmInstruction::DuplicateBytes => bytecode.push(ByteCodes::DuplicateBytes as u8),
                    AsmInstruction::Malloc => bytecode.push(ByteCodes::Malloc as u8),
                    AsmInstruction::Realloc => bytecode.push(ByteCodes::Realloc as u8),
                    AsmInstruction::Free => bytecode.push(ByteCodes::Free as u8),
                    AsmInstruction::Intr => bytecode.push(ByteCodes::Intr as u8),
                    AsmInstruction::IntrConst { code } => one_arg_number_instruction!(IntrConst, code, 1),
                    AsmInstruction::Exit => bytecode.push(ByteCodes::Exit as u8),
                    AsmInstruction::JumpConst { addr } => one_arg_address_instruction!(JumpConst, addr),
                    AsmInstruction::Jump => bytecode.push(ByteCodes::Jump as u8),
                    AsmInstruction::JumpNotZeroConst1 { addr } => one_arg_address_instruction!(JumpNotZeroConst1, addr),
                    AsmInstruction::JumpNotZeroConst2 { addr } => one_arg_address_instruction!(JumpNotZeroConst2, addr),
                    AsmInstruction::JumpNotZeroConst4 { addr } => one_arg_address_instruction!(JumpNotZeroConst4, addr),
                    AsmInstruction::JumpNotZeroConst8 { addr } => one_arg_address_instruction!(JumpNotZeroConst8, addr),
                    AsmInstruction::JumpNotZero1 => bytecode.push(ByteCodes::JumpNotZero1 as u8),
                    AsmInstruction::JumpNotZero2 => bytecode.push(ByteCodes::JumpNotZero2 as u8),
                    AsmInstruction::JumpNotZero4 => bytecode.push(ByteCodes::JumpNotZero4 as u8),
                    AsmInstruction::JumpNotZero8 => bytecode.push(ByteCodes::JumpNotZero8 as u8),
                    AsmInstruction::JumpZeroConst1 { addr } => one_arg_address_instruction!(JumpZeroConst1, addr),
                    AsmInstruction::JumpZeroConst2 { addr } => one_arg_address_instruction!(JumpZeroConst2, addr),
                    AsmInstruction::JumpZeroConst4 { addr } => one_arg_address_instruction!(JumpZeroConst4, addr),
                    AsmInstruction::JumpZeroConst8 { addr } => one_arg_address_instruction!(JumpZeroConst8, addr),
                    AsmInstruction::JumpZero1 => bytecode.push(ByteCodes::JumpZero1 as u8),
                    AsmInstruction::JumpZero2 => bytecode.push(ByteCodes::JumpZero2 as u8),
                    AsmInstruction::JumpZero4 => bytecode.push(ByteCodes::JumpZero4 as u8),
                    AsmInstruction::JumpZero8 => bytecode.push(ByteCodes::JumpZero8 as u8),
                    AsmInstruction::Nop => bytecode.push(ByteCodes::Nop as u8),
                }
            }
        };

    }

    // Fill in the unresolved symbols
    for label in unresolved_labels {
        
        let value = label_map.get(label.name).unwrap_or_else(
            || errors::undefined_symbol(&label.source, source, label.name)
        );

        bytecode[label.location.0..(label.location.0 + ADDRESS_SIZE)].copy_from_slice(&value.0.to_le_bytes());

    }

    // TODO: entry point

    bytecode.shrink_to_fit();
    bytecode
}

