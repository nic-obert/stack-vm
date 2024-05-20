use std::collections::HashMap;
use std::rc::Rc;

use crate::errors;
use crate::symbol_table::StaticValue;
use crate::{lang::AsmNode, symbol_table::SymbolTable};
use crate::lang::{AddressLike, AsmInstruction, AsmNodeValue, Number, NumberLike, ENTRY_SECTION_NAME};
use crate::tokenizer::{SourceCode, SourceToken};

use hivmlib::{ByteCodes, VirtualAddress, ADDRESS_SIZE, ERROR_CODE_SIZE, INSTRUCTION_SIZE, INTERRUPT_SIZE};


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

    let mut current_section: Option<&str> = None;

    macro_rules! push_op {
        ($op:ident) => {
            bytecode.push(ByteCodes::$op as u8)
        }
    }

    push_op!(JumpConst);
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
                label_map.insert(section, VirtualAddress(bytecode.len()));
                current_section = Some(section);
            },

            AsmNodeValue::Instruction(instruction) => {

                if current_section.is_none() {
                    errors::outside_section(&node.source, source, "Instructions must be located inside an assembly section");
                }


                macro_rules! one_arg_address_instruction {
                    ($name:ident, $addr:ident) => {{

                        bytecode.push(ByteCodes::$name as u8);

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

                        bytecode.extend_from_slice(&operand.to_le_bytes());
                    }}
                }

                macro_rules! one_arg_number_instruction {
                    ($name:ident, $value:ident, $size:expr) => {{

                        bytecode.push(ByteCodes::$name as u8);

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
                    AsmInstruction::AddInt1 => push_op!(AddInt1),
                    AsmInstruction::AddInt2 => push_op!(AddInt2),
                    AsmInstruction::AddInt4 => push_op!(AddInt4),
                    AsmInstruction::AddInt8 => push_op!(AddInt8),
                    AsmInstruction::SubInt1 => push_op!(SubInt1),
                    AsmInstruction::SubInt2 => push_op!(SubInt2),
                    AsmInstruction::SubInt4 => push_op!(SubInt4),
                    AsmInstruction::SubInt8 => push_op!(SubInt8),
                    AsmInstruction::MulInt1 => push_op!(MulInt1),
                    AsmInstruction::MulInt2 => push_op!(MulInt2),
                    AsmInstruction::MulInt4 => push_op!(MulInt4),
                    AsmInstruction::MulInt8 => push_op!(MulInt8),
                    AsmInstruction::DivInt1 => push_op!(DivInt1),
                    AsmInstruction::DivInt2 => push_op!(DivInt2),
                    AsmInstruction::DivInt4 => push_op!(DivInt4),
                    AsmInstruction::DivInt8 => push_op!(DivInt8),
                    AsmInstruction::ModInt1 => push_op!(ModInt1),
                    AsmInstruction::ModInt2 => push_op!(ModInt2),
                    AsmInstruction::ModInt4 => push_op!(ModInt4),
                    AsmInstruction::ModInt8 => push_op!(ModInt8),
                    AsmInstruction::AddFloat4 => push_op!(AddFloat4),
                    AsmInstruction::AddFloat8 => push_op!(AddFloat8),
                    AsmInstruction::SubFloat4 => push_op!(SubFloat4),
                    AsmInstruction::SubFloat8 => push_op!(SubFloat8),
                    AsmInstruction::MulFloat4 => push_op!(MulFloat4),
                    AsmInstruction::MulFloat8 => push_op!(MulFloat8),
                    AsmInstruction::DivFloat4 => push_op!(DivFloat4),
                    AsmInstruction::DivFloat8 => push_op!(DivFloat8),
                    AsmInstruction::ModFloat4 => push_op!(ModFloat4),
                    AsmInstruction::ModFloat8 => push_op!(ModFloat8),
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
                        
                        push_op!(LoadConstBytes);

                        bytecode.extend(bytes.len().to_le_bytes());

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

                        bytecode.extend_from_slice(value_bytes.as_slice());
                    },

                    AsmInstruction::Load1 => push_op!(Load1),
                    AsmInstruction::Load2 => push_op!(Load2),
                    AsmInstruction::Load4 => push_op!(Load4),
                    AsmInstruction::Load8 => push_op!(Load8),
                    AsmInstruction::LoadBytes => push_op!(LoadBytes),
                    AsmInstruction::VirtualConstToReal { addr } => one_arg_address_instruction!(VirtualConstToReal, addr),
                    AsmInstruction::VirtualToReal => push_op!(VirtualToReal),
                    AsmInstruction::Store1 => push_op!(Store1),
                    AsmInstruction::Store2 => push_op!(Store2),
                    AsmInstruction::Store4 => push_op!(Store4),
                    AsmInstruction::Store8 => push_op!(Store8),
                    AsmInstruction::StoreBytes => push_op!(StoreBytes),
                    AsmInstruction::Memmove1 => push_op!(Memmove1),
                    AsmInstruction::Memmove2 => push_op!(Memmove2),
                    AsmInstruction::Memmove4 => push_op!(Memmove4),
                    AsmInstruction::Memmove8 => push_op!(Memmove8),
                    AsmInstruction::MemmoveBytes => push_op!(MemmoveBytes),
                    AsmInstruction::Duplicate1 => push_op!(Duplicate1),
                    AsmInstruction::Duplicate2 => push_op!(Duplicate2),
                    AsmInstruction::Duplicate4 => push_op!(Duplicate4),
                    AsmInstruction::Duplicate8 => push_op!(Duplicate8),
                    AsmInstruction::DuplicateBytes => push_op!(DuplicateBytes),
                    AsmInstruction::Malloc => push_op!(Malloc),
                    AsmInstruction::Realloc => push_op!(Realloc),
                    AsmInstruction::Free => push_op!(Free),
                    AsmInstruction::Intr => push_op!(Intr),
                    AsmInstruction::IntrConst { value: code } => one_arg_number_instruction!(IntrConst, code, INTERRUPT_SIZE),
                    AsmInstruction::ReadError => push_op!(ReadError),
                    AsmInstruction::SetError => push_op!(SetError),
                    AsmInstruction::SetErrorConst { value } => one_arg_number_instruction!(SetErrorConst, value, ERROR_CODE_SIZE),
                    AsmInstruction::Exit => push_op!(Exit),
                    AsmInstruction::JumpConst { addr } => one_arg_address_instruction!(JumpConst, addr),
                    AsmInstruction::Jump => push_op!(Jump),
                    AsmInstruction::JumpNotZeroConst1 { addr } => one_arg_address_instruction!(JumpNotZeroConst1, addr),
                    AsmInstruction::JumpNotZeroConst2 { addr } => one_arg_address_instruction!(JumpNotZeroConst2, addr),
                    AsmInstruction::JumpNotZeroConst4 { addr } => one_arg_address_instruction!(JumpNotZeroConst4, addr),
                    AsmInstruction::JumpNotZeroConst8 { addr } => one_arg_address_instruction!(JumpNotZeroConst8, addr),
                    AsmInstruction::JumpNotZero1 => push_op!(JumpNotZero1),
                    AsmInstruction::JumpNotZero2 => push_op!(JumpNotZero2),
                    AsmInstruction::JumpNotZero4 => push_op!(JumpNotZero4),
                    AsmInstruction::JumpNotZero8 => push_op!(JumpNotZero8),
                    AsmInstruction::JumpZeroConst1 { addr } => one_arg_address_instruction!(JumpZeroConst1, addr),
                    AsmInstruction::JumpZeroConst2 { addr } => one_arg_address_instruction!(JumpZeroConst2, addr),
                    AsmInstruction::JumpZeroConst4 { addr } => one_arg_address_instruction!(JumpZeroConst4, addr),
                    AsmInstruction::JumpZeroConst8 { addr } => one_arg_address_instruction!(JumpZeroConst8, addr),
                    AsmInstruction::JumpZero1 => push_op!(JumpZero1),
                    AsmInstruction::JumpZero2 => push_op!(JumpZero2),
                    AsmInstruction::JumpZero4 => push_op!(JumpZero4),
                    AsmInstruction::JumpZero8 => push_op!(JumpZero8),
                    AsmInstruction::JumpError => push_op!(JumpError),
                    AsmInstruction::JumpNoError => push_op!(JumpNoError),
                    AsmInstruction::JumpErrorConst { addr } => one_arg_address_instruction!(JumpErrorConst, addr),
                    AsmInstruction::JumpNoErrorConst { addr } => one_arg_address_instruction!(JumpNoErrorConst, addr),
                    
                    AsmInstruction::DefineNumber { size, value } => {
                        
                        let number_size = match size.0 {
                            NumberLike::Number(n, _size)
                            => if let Number::Uint(n) = n {
                                n as usize
                            } else {
                                errors::invalid_argument(&size.1, source, "Expected an unsigned integer as number size.");
                            },
                            NumberLike::CurrentPosition => bytecode.len(),
                            NumberLike::Symbol(_)
                                => errors::invalid_argument(&size.1, source, "Cannot use a symbol as number size in this context. Only literals are allowed."),
                        };

                        let mut number_value = match value.0 {
                            NumberLike::Number(n, s) => n.as_le_bytes()[..s as usize].to_vec(),
                            NumberLike::CurrentPosition => bytecode.len().to_le_bytes().to_vec(),
                            NumberLike::Symbol(_)
                                => errors::invalid_argument(&value.1, source, "Expected a numeric literal.")
                        };

                        if number_value.len() > number_size {
                            errors::invalid_argument(&value.1, source, format!("Expected a number of size {}, but got a number of size {}", number_size, number_value.len()).as_str());
                        }

                        // Make the number value exactly the requested size
                        if number_value.len() < number_size {
                            number_value.extend(vec![0; number_size - number_value.len()]);
                        }
                    
                        bytecode.extend(number_value);
                    },

                    AsmInstruction::DefineBytes { bytes } => {

                        let mut value_bytes = Vec::with_capacity(bytes.len());
                        
                        for byte in bytes {

                            let bytes_per_byte = match &byte.0 {
                                
                                NumberLike::Number(n, _size) => {
                                    if matches!(n, Number::Float(_)) {
                                        errors::invalid_argument(&byte.1, source, format!("Invalid constant value `{:?}`. Must be an integer, not a float.", n).as_str())
                                    }

                                    n.as_le_bytes()
                                },
                                
                                NumberLike::CurrentPosition => bytecode.len().to_le_bytes().to_vec(),

                                NumberLike::Symbol(_) 
                                    => errors::invalid_argument(&byte.1, source, "Expected a byte literal")
                            };

                            if bytes_per_byte.len() != 1 {
                                errors::invalid_argument(&byte.1, source, format!("Invalid constant value `{:?}`. Must be a single byte but {} bytes were provided.", byte.0, bytes_per_byte.len()).as_str())
                            }

                            value_bytes.push(bytes_per_byte[0]);
                        }

                        bytecode.extend(value_bytes);
                    },

                    AsmInstruction::DefineString { string } => {

                        let string = match symbol_table.get_static(string) {
                            StaticValue::StringLiteral(string) => string,
                        };

                        bytecode.extend(string.as_bytes());
                    },

                    AsmInstruction::Nop => push_op!(Nop),
                }
            }
        };

    }

    // Fill in the unresolved symbols
    for label in unresolved_labels {
        
        let value = label_map.get(label.name).unwrap_or_else(
            || errors::undefined_symbol(&label.source, source)
        );

        bytecode[label.location.0..(label.location.0 + ADDRESS_SIZE)].copy_from_slice(&value.0.to_le_bytes());

    }

    // Fill in the entry point address
    if let Some(entry) = label_map.get(ENTRY_SECTION_NAME) {
        bytecode[1..(1 + ADDRESS_SIZE)].copy_from_slice(&entry.0.to_le_bytes());
    }

    bytecode.shrink_to_fit();
    bytecode
}

