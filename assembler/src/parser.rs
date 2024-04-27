use hivmlib::ByteCodes;

use crate::tokenizer::{SourceCode, Token, TokenList, TokenValue};
use crate::symbol_table::SymbolTable;
use crate::lang::{AsmInstruction, AsmNode, AsmSection, AsmValue, Number, NumberLike, AddressLike};
use crate::errors;


// fn get_highest_priority(tokens: &TokenList) -> Option<usize> {
    
//     let mut highest_priority = TokenBasePriority::None as TokenPriority;
//     let mut highest_priority_index = None;

//     for (index, token) in tokens.iter().enumerate() {
//         if token.priority > highest_priority {
//             highest_priority = token.priority;
//             highest_priority_index = Some(index);
//         }
//     }

//     highest_priority_index
// }


fn parse_operands<'a>(tokens: &'a [Token<'a>], symbol_table: &SymbolTable, source: SourceCode) -> Vec<AsmValue> {
    
    // TODO: implement in-line constant math and eventual in-line operators.

    // Allocate the maximum capacity needed for the operands. Since most operations will not contain
    // in-line operations, this will avoid reallocations and space won't be wasted for most cases.
    let mut operands: Vec<AsmValue> = Vec::with_capacity(tokens.len());

    // The parsing occurs on a left-to-right manner for now.

    let mut i = 0;

    while let Some(token) = tokens.get(i) {

        match &token.value {

            TokenValue::StringLiteral(s) => operands.push(AsmValue::StringLiteral(*s)),
            TokenValue::CharLiteral(ch) => operands.push(AsmValue::Const(Number::Uint(*ch as u64))),
            TokenValue::Number(n) => operands.push(AsmValue::Const(n.clone())),

            TokenValue::Identifier(id) => operands.push(AsmValue::Symbol(*id)),

            TokenValue::Dollar => operands.push(AsmValue::CurrentPosition),
            
            TokenValue::Mod => {
                i += 1;

                let next_token = tokens.get(i).unwrap_or_else(
                    || errors::parsing_error(&token.source, source, "Missing symbol name after `%`."));

                let symbol_id = if let TokenValue::Identifier(id) = next_token.value {
                    id
                } else {
                    errors::parsing_error(&next_token.source, source, "Expected a symbol name after `%`.")
                };

                let symbol_value = symbol_table.get_symbol(symbol_id).unwrap_or_else(
                    || errors::parsing_error(&next_token.source, source, "Use of undefined or undeclared symbol."))
                    .borrow()
                    .value
                    .clone()
                    .unwrap_or_else(
                        || errors::parsing_error(&next_token.source, source, "Symbol has no value."));

                operands.push(symbol_value);              
            },
            
            TokenValue::Plus => todo!(),
            TokenValue::Minus => todo!(),
            TokenValue::Star => todo!(),
            TokenValue::Div => todo!(),
            
            TokenValue::Instruction(_) |
            TokenValue::Dot |
            TokenValue::At |
            TokenValue::Colon 
                => errors::parsing_error(&token.source, source, "Token cannot be used as an operand.")
        }

        i += 1;
    }

    operands.shrink_to_fit();
    operands
}


pub fn parse<'a>(token_lines: &'a [TokenList<'a>], source: SourceCode, symbol_table: &'a SymbolTable<'a>) -> Vec<AsmNode<'a>> {

    // A good estimate for the number of nodes is the number of assembly lines. This is because an assembly line 
    // usually translates to a single instruction. This should avoid reallocations in most cases.
    let mut nodes = Vec::with_capacity(token_lines.len());

    let mut i: usize = 0;

    macro_rules! next_line {
        () => {
            i += 1;
            continue;
        };
    }

    while let Some(line) = &token_lines.get(i) {

        // Assume the line is not empty since the lexer has already filtered out empty lines

        let main_operator = &line[0];

        let operands = parse_operands(&line[1..], symbol_table, source);

        match main_operator.value {

            TokenValue::At => {
                if operands.len() != 1 {
                    errors::parsing_error(&main_operator.source, source, "Operator expects exactly one argument.");
                }

                let symbol_id = if let AsmValue::Symbol(id) = operands[0] {
                    id
                } else {
                    errors::parsing_error(&main_operator.source, source, "Expected a symbol as label name.");
                };

                let symbol = symbol_table.get_symbol(symbol_id).unwrap().borrow();

                // Disallow defining a label more than once.
                if symbol.value.is_some() {
                    errors::symbol_redeclaration(&main_operator.source, source, &symbol);
                }

                nodes.push(AsmNode::Label(symbol.source.string));
                
                // Mark the label as declared at this location in the source code. No further declarations of the same label are allowed.
                symbol_table.define_symbol(symbol_id, AsmValue::Symbol(symbol_id), main_operator.source.clone());
            },

            TokenValue::Instruction(code) => {

                macro_rules! no_args_instruction {
                    ($name:ident) => {{
                        if !operands.is_empty() {
                            errors::parsing_error(&main_operator.source, source, "Operator expects no arguments.");
                        }
                        nodes.push(AsmNode::Instruction(AsmInstruction::$name));
                    }}
                }

                macro_rules! one_arg_numeric_instruction {
                    ($name:ident) => {{
                        if operands.len() != 1 {
                            errors::parsing_error(&main_operator.source, source, "Operator expects exactly one argument.");
                        }

                        let val = match &operands[0] {
                            AsmValue::Const(n) => NumberLike::Number(n.clone()),
                            AsmValue::CurrentPosition => NumberLike::CurrentPosition,
                            AsmValue::Symbol(id) => NumberLike::Symbol(*id),
                           
                            AsmValue::StringLiteral(_)
                                => errors::parsing_error(&main_operator.source, source, "Expected a numeric value, got a string literal."),
                        };

                        nodes.push(AsmNode::Instruction(AsmInstruction::$name { value: val }));
                    }}
                }

                macro_rules! one_arg_address_instruction {
                    ($name:ident) => {{
                        if operands.len() != 1 {
                            errors::parsing_error(&main_operator.source, source, "Operator expects exactly one argument.");
                        }

                        let val = match &operands[0] {
                            AsmValue::Const(n) => AddressLike::Number(n.clone()),
                            AsmValue::CurrentPosition => AddressLike::CurrentPosition,
                            AsmValue::Symbol(id) => AddressLike::Symbol(*id),
                           
                            AsmValue::StringLiteral(_)
                                => errors::parsing_error(&main_operator.source, source, "Expected an address value, got a string literal."),
                        };

                        nodes.push(AsmNode::Instruction(AsmInstruction::$name { addr: val }));
                    }}
                }

                match code {
                    ByteCodes::AddInt1 => no_args_instruction!(AddInt1),
                    ByteCodes::AddInt2 => no_args_instruction!(AddInt2),
                    ByteCodes::AddInt4 => no_args_instruction!(AddInt4),
                    ByteCodes::AddInt8 => no_args_instruction!(AddInt8),
                    ByteCodes::SubInt1 => no_args_instruction!(SubInt1),
                    ByteCodes::SubInt2 => no_args_instruction!(SubInt2),
                    ByteCodes::SubInt4 => no_args_instruction!(SubInt4),
                    ByteCodes::SubInt8 => no_args_instruction!(SubInt8),
                    ByteCodes::MulInt1 => no_args_instruction!(MulInt1),
                    ByteCodes::MulInt2 => no_args_instruction!(MulInt2),
                    ByteCodes::MulInt4 => no_args_instruction!(MulInt4),
                    ByteCodes::MulInt8 => no_args_instruction!(MulInt8),
                    ByteCodes::DivInt1 => no_args_instruction!(DivInt1),
                    ByteCodes::DivInt2 => no_args_instruction!(DivInt2),
                    ByteCodes::DivInt4 => no_args_instruction!(DivInt4),
                    ByteCodes::DivInt8 => no_args_instruction!(DivInt8),
                    ByteCodes::ModInt1 => no_args_instruction!(ModInt1),
                    ByteCodes::ModInt2 => no_args_instruction!(ModInt2),
                    ByteCodes::ModInt4 => no_args_instruction!(ModInt4),
                    ByteCodes::ModInt8 => no_args_instruction!(ModInt8),
                    ByteCodes::AddFloat4 => no_args_instruction!(AddFloat4),
                    ByteCodes::AddFloat8 => no_args_instruction!(AddFloat8),
                    ByteCodes::SubFloat4 => no_args_instruction!(SubFloat4),
                    ByteCodes::SubFloat8 => no_args_instruction!(SubFloat8),
                    ByteCodes::MulFloat4 => no_args_instruction!(MulFloat4),
                    ByteCodes::MulFloat8 => no_args_instruction!(MulFloat8),
                    ByteCodes::DivFloat4 => no_args_instruction!(DivFloat4),
                    ByteCodes::DivFloat8 => no_args_instruction!(DivFloat8),
                    ByteCodes::ModFloat4 => no_args_instruction!(ModFloat4),
                    ByteCodes::ModFloat8 => no_args_instruction!(ModFloat8),
                    ByteCodes::LoadStatic1 => one_arg_address_instruction!(LoadStatic1),
                    ByteCodes::LoadStatic2 => one_arg_address_instruction!(LoadStatic2),
                    ByteCodes::LoadStatic4 => one_arg_address_instruction!(LoadStatic4),
                    ByteCodes::LoadStatic8 => one_arg_address_instruction!(LoadStatic8),
                    ByteCodes::LoadStaticBytes => one_arg_address_instruction!(LoadStaticBytes),
                    ByteCodes::LoadConst1 => one_arg_numeric_instruction!(LoadConst1),
                    ByteCodes::LoadConst2 => one_arg_numeric_instruction!(LoadConst2),
                    ByteCodes::LoadConst4 => one_arg_numeric_instruction!(LoadConst4),
                    ByteCodes::LoadConst8 => one_arg_numeric_instruction!(LoadConst8),
                    ByteCodes::LoadConstBytes => {
                        let mut bytes = Vec::with_capacity(operands.len());

                        for op in operands {
                            let val = match op {
                                AsmValue::Const(n) => NumberLike::Number(n),
                                AsmValue::CurrentPosition => NumberLike::CurrentPosition,
                                AsmValue::Symbol(id) => NumberLike::Symbol(id),

                                AsmValue::StringLiteral(_)
                                    => errors::parsing_error(&main_operator.source, source, "Expected a byte value, got a string literal."),
                            };

                            bytes.push(val);
                        }

                        nodes.push(AsmNode::Instruction(AsmInstruction::LoadConstBytes { bytes }));
                    },
                    ByteCodes::Load1 => no_args_instruction!(Load1),
                    ByteCodes::Load2 => no_args_instruction!(Load2),
                    ByteCodes::Load4 => no_args_instruction!(Load4),
                    ByteCodes::Load8 => no_args_instruction!(Load8),
                    ByteCodes::LoadBytes => no_args_instruction!(LoadBytes),
                    ByteCodes::VirtualConstToReal => one_arg_address_instruction!(VirtualConstToReal),
                    ByteCodes::VirtualToReal => no_args_instruction!(VirtualToReal),
                    ByteCodes::Store1 => no_args_instruction!(Store1),
                    ByteCodes::Store2 => no_args_instruction!(Store2),
                    ByteCodes::Store4 => no_args_instruction!(Store4),
                    ByteCodes::Store8 => no_args_instruction!(Store8),
                    ByteCodes::StoreBytes => no_args_instruction!(StoreBytes),
                    ByteCodes::Memmove1 => no_args_instruction!(Memmove1),
                    ByteCodes::Memmove2 => no_args_instruction!(Memmove2),
                    ByteCodes::Memmove4 => no_args_instruction!(Memmove4),
                    ByteCodes::Memmove8 => no_args_instruction!(Memmove8),
                    ByteCodes::MemmoveBytes => no_args_instruction!(MemmoveBytes),
                    ByteCodes::Duplicate1 => no_args_instruction!(Duplicate1),
                    ByteCodes::Duplicate2 => no_args_instruction!(Duplicate2),
                    ByteCodes::Duplicate4 => no_args_instruction!(Duplicate4),
                    ByteCodes::Duplicate8 => no_args_instruction!(Duplicate8),
                    ByteCodes::DuplicateBytes => no_args_instruction!(DuplicateBytes),
                    ByteCodes::Malloc => no_args_instruction!(Malloc),
                    ByteCodes::Realloc => no_args_instruction!(Realloc),
                    ByteCodes::Free => no_args_instruction!(Free),
                    ByteCodes::Intr => no_args_instruction!(Intr),
                    ByteCodes::IntrConst => {
                        if operands.len() != 1 {
                            errors::parsing_error(&main_operator.source, source, "Operator expects exactly one argument.");
                        }

                        let val = match &operands[0] {
                            AsmValue::Const(n) => NumberLike::Number(n.clone()),
                            AsmValue::CurrentPosition => NumberLike::CurrentPosition,
                            AsmValue::Symbol(id) => NumberLike::Symbol(*id),
                           
                            AsmValue::StringLiteral(_)
                                => errors::parsing_error(&main_operator.source, source, "Expected a numeric value, got a string literal."),
                        };

                        nodes.push(AsmNode::Instruction(AsmInstruction::IntrConst { code: val }));
                    },
                    ByteCodes::Exit => no_args_instruction!(Exit),
                    ByteCodes::JumpConst => one_arg_address_instruction!(JumpConst),
                    ByteCodes::Jump => no_args_instruction!(Jump),
                    ByteCodes::JumpNotZeroConst1 => one_arg_address_instruction!(JumpNotZeroConst1),
                    ByteCodes::JumpNotZeroConst2 => one_arg_address_instruction!(JumpNotZeroConst2),
                    ByteCodes::JumpNotZeroConst4 => one_arg_address_instruction!(JumpNotZeroConst4),
                    ByteCodes::JumpNotZeroConst8 => one_arg_address_instruction!(JumpNotZeroConst8),
                    ByteCodes::JumpNotZero1 => no_args_instruction!(JumpNotZero1),
                    ByteCodes::JumpNotZero2 => no_args_instruction!(JumpNotZero2),
                    ByteCodes::JumpNotZero4 => no_args_instruction!(JumpNotZero4),
                    ByteCodes::JumpNotZero8 => no_args_instruction!(JumpNotZero8),
                    ByteCodes::JumpZeroConst1 => one_arg_address_instruction!(JumpZeroConst1),
                    ByteCodes::JumpZeroConst2 => one_arg_address_instruction!(JumpZeroConst2),
                    ByteCodes::JumpZeroConst4 => one_arg_address_instruction!(JumpZeroConst4),
                    ByteCodes::JumpZeroConst8 => one_arg_address_instruction!(JumpZeroConst8),
                    ByteCodes::JumpZero1 => no_args_instruction!(JumpZero1),
                    ByteCodes::JumpZero2 => no_args_instruction!(JumpZero2),
                    ByteCodes::JumpZero4 => no_args_instruction!(JumpZero4),
                    ByteCodes::JumpZero8 => no_args_instruction!(JumpZero8),
                    ByteCodes::Nop => no_args_instruction!(Nop),
                }
            },

            TokenValue::Mod => todo!(),

            TokenValue::Dot => {
                if operands.len() != 1 {
                    errors::parsing_error(&main_operator.source, source, "Operator expects exactly one argument.");
                }

                let symbol_id = if let AsmValue::Symbol(id) = operands[0] {
                    id
                } else {
                    errors::parsing_error(&main_operator.source, source, "Expected a symbol as section name.");
                };

                let symbol = symbol_table.get_symbol(symbol_id).unwrap().borrow();

                if symbol.value.is_some() {
                    errors::symbol_redeclaration(&main_operator.source, source, &symbol);
                }
                
                nodes.push(AsmNode::Section(AsmSection::from_name(symbol.source.string)));                
                
                // Mark this section name as declared at this source code location.
                symbol_table.define_symbol(symbol_id, AsmValue::Symbol(symbol_id), main_operator.source.clone());
            },
            
            TokenValue::Number(_) |
            TokenValue::Identifier(_) |
            TokenValue::Dollar |
            TokenValue::Plus |
            TokenValue::Minus |
            TokenValue::Star |
            TokenValue::Div |
            TokenValue::StringLiteral(_) |
            TokenValue::CharLiteral(_) |
            TokenValue::Colon
                => errors::parsing_error(&main_operator.source, source, "Token cannot be used as a main operator.")
        }
        
        next_line!();
    }

    nodes.shrink_to_fit();
    nodes
}
