use std::collections::HashMap;

use hivmlib::ByteCodes;

use crate::tokenizer::{SourceCode, Token, TokenList, TokenValue};
use crate::symbol_table::{SymbolID, SymbolTable};
use crate::lang::{AddressLike, AsmInstruction, AsmNode, AsmNodeValue, AsmOperand, AsmValue, Number, NumberLike, PseudoInstructions};
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


fn parse_operands<'a>(tokens: &'a [Token<'a>], symbol_table: &SymbolTable, source: SourceCode) -> Box<[AsmOperand<'a>]> {
    
    // TODO: implement in-line constant math and eventual in-line operators.

    // Allocate the maximum capacity needed for the operands. Since most operations will not contain
    // in-line operations, this will avoid reallocations and space won't be wasted for most cases.
    let mut operands = Vec::with_capacity(tokens.len());

    // The parsing occurs on a left-to-right manner for now.

    let mut i = 0;

    while let Some(token) = tokens.get(i) {

        macro_rules! push_op {
            ($op:expr, $source:expr) => {
                operands.push(AsmOperand { value: $op, source: $source.source.clone() })
            }
        }

        match &token.value {

            TokenValue::StringLiteral(s) => push_op!(AsmValue::StringLiteral(*s), token),
            TokenValue::CharLiteral(ch) => push_op!(AsmValue::Const(Number::Uint(*ch as u64)), token),
            TokenValue::Number(n) => push_op!(AsmValue::Const(n.clone()), token),

            TokenValue::Identifier(id) => push_op!(AsmValue::Symbol(*id), token),

            TokenValue::Dollar => push_op!(AsmValue::CurrentPosition, token),
            
            TokenValue::Bang => {
                i += 1;

                let next_token = tokens.get(i).unwrap_or_else(
                    || errors::parsing_error(&token.source, source, "Missing macro name after `!`."));

                let symbol_id = if let TokenValue::Identifier(id) = next_token.value {
                    id
                } else {
                    errors::parsing_error(&next_token.source, source, "Expected a macro name after `!`.")
                };

                let symbol_value = symbol_table.get_symbol(symbol_id)
                    .borrow()
                    .value
                    .clone()
                    .unwrap_or_else(
                        || errors::parsing_error(&next_token.source, source, "Macro has no value."));

                push_op!(symbol_value, token);              
            },

            TokenValue::Mod => {
                i += 1;

                let next_token = tokens.get(i).unwrap_or_else(
                    || errors::parsing_error(&token.source, source, "Missing macro parameter name after `%`."));

                let symbol_id = if let TokenValue::Identifier(id) = next_token.value {
                    id
                } else {
                    errors::parsing_error(&next_token.source, source, "Expected a macro parameter name after `%`.")
                };

                push_op!(AsmValue::MacroParameter(symbol_id), token);
            },
            
            TokenValue::Plus => todo!(),
            TokenValue::Minus => todo!(),
            TokenValue::Star => todo!(),
            TokenValue::Div => todo!(),
            
            TokenValue::Instruction(_) |
            TokenValue::PseudoInstruction(_) |
            TokenValue::Dot |
            TokenValue::At |
            TokenValue::Colon |
            TokenValue::EndMacro |
            TokenValue::ValueMacroDef
                => errors::parsing_error(&token.source, source, "Token cannot be used as an operand.")
        }

        i += 1;
    }

    operands.into_boxed_slice()
}


struct MacroDef<'a> {
    args: Box<[SymbolID]>,
    body: Box<[(&'a Token<'a>, Box<[AsmOperand<'a>]>)]>
}


type MacroMap<'a> = HashMap<SymbolID, MacroDef<'a>>;


fn parse_line<'a>(main_operator: &'a Token<'a>, operands: Box<[AsmOperand<'a>]>, nodes: &mut Vec<AsmNode<'a>>, macros: &mut MacroMap<'a>, token_lines: &'a [TokenList<'a>], line_index: &mut usize, source: SourceCode, symbol_table: &'a SymbolTable<'a>) {

    macro_rules! check_arg_count {
        ($required:expr) => {
            if operands.len() != $required {
                errors::parsing_error(&main_operator.source, source, format!("Operator expects exactly {} arguments, but {} were given.", $required, operands.len()).as_str())
            }
        }
    }

    // Handle an eventual macro call and expand it
    if matches!(main_operator.value, TokenValue::Bang) {
        let macro_name = operands.first().unwrap_or_else(
            || errors::parsing_error(&main_operator.source, source, "Macro call must have a macro name after the !.")
        );

        let macro_id = if let AsmValue::Symbol(id) = macro_name.value {
            id
        } else {
            errors::parsing_error(&macro_name.source, source, "Expected a symbol as macro name.")
        };

        let macro_def = macros.get(&macro_id).unwrap_or_else(
            || errors::undefined_symbol(&macro_name.source, source));

        // Skip the macro name
        let operands = &operands[1..];

        check_arg_count!(macro_def.args.len());

        let mut macro_args = HashMap::new();

        for (arg, param) in operands.iter().zip(macro_def.args.iter()) {
            macro_args.insert(*param, arg);
        }

        // Expand the macro

        let mut expanded_macro = Vec::with_capacity(macro_def.body.len());

        // TODO: maybe eventually use a Cow to avoid useless cloning
        for (body_line_main_operator, raw_body_line_operands) in macro_def.body.iter() {
            
            let mut body_line_operands: Vec<AsmOperand> = Vec::with_capacity(raw_body_line_operands.len());
            for op in raw_body_line_operands.iter() {
                body_line_operands.push(
                    if let AsmValue::MacroParameter(id) = op.value {
                        (*macro_args.get(&id).unwrap_or_else(
                            || errors::parsing_error(&op.source, source, "Macro symbol not found.")
                        )).clone()
                    } else {
                        op.clone()
                    }
                )
            }

            expanded_macro.push((*body_line_main_operator, body_line_operands.into_boxed_slice()));
        }

        // Once the macro is expanded, parse it normally
        for (main_operator, operands) in expanded_macro {
            parse_line(main_operator, operands, nodes, macros, token_lines, line_index, source, symbol_table);
        }

        // The macro has been expanded and parsed, there's nothing more to do
        return;
    }


    macro_rules! no_args_instruction {
        ($name:ident) => {{
            check_arg_count!(0);
            nodes.push(AsmNode { 
                value: AsmNodeValue::Instruction(AsmInstruction::$name),
                source: main_operator.source.clone()
            });
        }}
    }

    macro_rules! parse_numeric_arg {
        ($index:literal, $op:ident, $value:ident) => {
            let $op = &operands[$index];
            parse_numeric_arg!($op, $value);
        };
        ($op:ident, $value:ident) => {
            let $value = match &$op.value {
                AsmValue::Const(n) => NumberLike::from_number(n),
                AsmValue::CurrentPosition => NumberLike::CurrentPosition,
                AsmValue::Symbol(id) => NumberLike::Symbol(*id),
               
                AsmValue::StringLiteral(_)
                    => errors::parsing_error(&$op.source, source, "Expected a numeric value, got a string literal."),

                AsmValue::MacroParameter(_)
                    => errors::parsing_error(&$op.source, source, "Macro parameter outside of a macro definition.")
            };
        };
    }

    macro_rules! one_arg_numeric_instruction {
        ($name:ident) => {{
            check_arg_count!(1);

            parse_numeric_arg!(0, op, val);

            nodes.push(AsmNode { 
                value: AsmNodeValue::Instruction(AsmInstruction::$name { value: (val, op.source.clone()) }),
                source: main_operator.source.clone()
            });
        }}
    }

    macro_rules! parse_address_arg {
        ($index:literal, $op:ident, $value:ident) => {
            let $op = &operands[0];
            let $value = match &$op.value {
                AsmValue::Const(n) => AddressLike::Number(n.clone()),
                AsmValue::CurrentPosition => AddressLike::CurrentPosition,
                AsmValue::Symbol(id) => AddressLike::Symbol(*id),
               
                AsmValue::StringLiteral(_)
                    => errors::parsing_error(&$op.source, source, "Expected an address value, got a string literal."),

                AsmValue::MacroParameter(_)
                    => errors::parsing_error(&$op.source, source, "Macro parameter outside of a macro definition.")
            };
        };
    }

    macro_rules! one_arg_address_instruction {
        ($name:ident) => {{
            check_arg_count!(1);

            parse_address_arg!(0, op, val);

            nodes.push(AsmNode {
                value: AsmNodeValue::Instruction(AsmInstruction::$name { addr: (val, op.source.clone()) }),
                source: main_operator.source.clone()
            });
        }}
    }

    match main_operator.value {

        TokenValue::At => {
            check_arg_count!(1);

            let op = &operands[0];
            let symbol_id = if let AsmValue::Symbol(id) = op.value {
                id
            } else {
                errors::parsing_error(&op.source, source, "Expected a symbol as label name.");
            };

            // Reduce the scope of `symbol` to comply with the dynamic borrow checker
            {
                let symbol = symbol_table.get_symbol(symbol_id).borrow();

                // Disallow defining a label more than once.
                if symbol.value.is_some() {
                    errors::symbol_redeclaration(&op.source, source, &symbol);
                }

                nodes.push(AsmNode { 
                    value: AsmNodeValue::Label(symbol.source.string),
                    source: main_operator.source.clone()
                });
            }
            
            // Mark the label as declared at this location in the source code. No further declarations of the same label are allowed.
            // Leave the value as None since its location in the binary is not known yet. It will be resolved when the binary is generated.
            symbol_table.define_symbol(symbol_id, None, op.source.clone());
        },

        TokenValue::ValueMacroDef => {

            // Syntax: %= macro_name value

            let macro_name = operands.first().unwrap_or_else(
                || errors::parsing_error(&main_operator.source, source, "Macro value declaration must have a name after the %=.")
            );

            let macro_id = if let AsmValue::Symbol(id) = macro_name.value {
                id
            } else {
                errors::parsing_error(&macro_name.source, source, "Expected a symbol as macro name.")
            };

            check_arg_count!(2);

            // Assume it's present because of the previous length check
            let value = operands[1].value.clone();

            symbol_table.define_symbol(macro_id, Some(value), operands[1].source.clone())

        },

        TokenValue::Mod => {

            // Syntax: %macro_name arg1, arg2, arg3, ...
            // ...
            // %endmacro


            let macro_name = operands.first().unwrap_or_else(
                || errors::parsing_error(&main_operator.source, source, "Macro declaration must have a name after the %.")
            );

            let macro_id = if let AsmValue::Symbol(id) = macro_name.value {
                id
            } else {
                errors::parsing_error(&macro_name.source, source, "Expected a symbol as macro name.")
            };

            let params: Vec<SymbolID> = operands[1..].iter().map(|op| {
                if let AsmValue::Symbol(id) = op.value {
                    id
                } else {
                    errors::parsing_error(&op.source, source, "Expected a symbol as macro parametr.")
                }
            }).collect();

            // Get the macro body

            let mut body = Vec::new();
            
            loop {

                *line_index += 1;

                let line = token_lines.get(*line_index).unwrap_or_else(
                    || errors::parsing_error(&main_operator.source, source, "Macro definition must end with `%endmacro`."));
                
                let body_main_operator = &line[0];
                let body_operands = parse_operands(&line[1..], symbol_table, source);
                
                if matches!(body_main_operator.value, TokenValue::EndMacro) {
                    break;
                }

                body.push((body_main_operator, body_operands));

            }

            if macros.insert(macro_id, MacroDef {
                args: params.into_boxed_slice(),
                body: body.into_boxed_slice()
            }).is_some() {
                // Disallow redefining a macro.
                errors::symbol_redeclaration(&main_operator.source, source, &symbol_table.get_symbol(macro_id).borrow());
            }
        },

        TokenValue::Dot => {
            check_arg_count!(1);

            let op = &operands[0];
            let symbol_id = if let AsmValue::Symbol(id) = op.value {
                id
            } else {
                errors::parsing_error(&op.source, source, "Expected a symbol as section name.");
            };

            { // Scope for symbol borrow (cannot borrow again later while `symbol` is still borrowed)
                let symbol = symbol_table.get_symbol(symbol_id).borrow();

                if symbol.value.is_some() {
                    errors::symbol_redeclaration(&op.source, source, &symbol);
                }
                
                nodes.push(AsmNode {
                    value: AsmNodeValue::Section(symbol.source.string),
                    source: main_operator.source.clone()
                });
            }
            
            // Mark this section name as declared at this source code location.
            symbol_table.define_symbol(symbol_id, None, main_operator.source.clone());
        },

        TokenValue::Instruction(code) => match code {

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

            ByteCodes::LoadStaticBytes => {
                check_arg_count!(2);

                parse_address_arg!(0, addr_op, addr);
                parse_numeric_arg!(1, count_op, count);

                nodes.push(AsmNode {
                    value: AsmNodeValue::Instruction(AsmInstruction::LoadStaticBytes { addr: (addr, addr_op.source.clone()), count: (count, count_op.source.clone()) }),
                    source: main_operator.source.clone()
                });
            },

            ByteCodes::LoadConst1 => one_arg_numeric_instruction!(LoadConst1),
            ByteCodes::LoadConst2 => one_arg_numeric_instruction!(LoadConst2),
            ByteCodes::LoadConst4 => one_arg_numeric_instruction!(LoadConst4),
            ByteCodes::LoadConst8 => one_arg_numeric_instruction!(LoadConst8),

            ByteCodes::LoadConstBytes => {
                let mut bytes = Vec::with_capacity(operands.len());

                for op in operands.iter() {
                    parse_numeric_arg!(op, val);
                    bytes.push((val, op.source.clone()));
                }

                nodes.push(AsmNode {
                    value: AsmNodeValue::Instruction(AsmInstruction::LoadConstBytes { bytes }),
                    source: main_operator.source.clone()
                });
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
            ByteCodes::IntrConst => one_arg_numeric_instruction!(IntrConst),
            ByteCodes::ReadError => no_args_instruction!(ReadError),
            ByteCodes::SetErrorConst => one_arg_numeric_instruction!(SetErrorConst),
            ByteCodes::SetError => no_args_instruction!(SetError),
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
            ByteCodes::JumpErrorConst => one_arg_address_instruction!(JumpErrorConst),
            ByteCodes::JumpNoErrorConst => one_arg_address_instruction!(JumpNoErrorConst),
            ByteCodes::JumpZero1 => no_args_instruction!(JumpZero1),
            ByteCodes::JumpZero2 => no_args_instruction!(JumpZero2),
            ByteCodes::JumpZero4 => no_args_instruction!(JumpZero4),
            ByteCodes::JumpZero8 => no_args_instruction!(JumpZero8),
            ByteCodes::JumpError => no_args_instruction!(JumpError),
            ByteCodes::JumpNoError => no_args_instruction!(JumpNoError),
            ByteCodes::Nop => no_args_instruction!(Nop),
        },

        TokenValue::PseudoInstruction(instruction) => match instruction {

            PseudoInstructions::DefineNumber => {
                check_arg_count!(2);

                parse_numeric_arg!(0, size_op, size);
                parse_numeric_arg!(1, value_op, value);

                nodes.push(AsmNode {
                    value: AsmNodeValue::Instruction(AsmInstruction::DefineNumber { 
                        size: (size, size_op.source.clone()), 
                        value: (value, value_op.source.clone()) 
                    }),
                    source: main_operator.source.clone()
                });
            },

            PseudoInstructions::DefineBytes => {
                let mut bytes = Vec::with_capacity(operands.len());

                for op in operands.iter() {
                    parse_numeric_arg!(op, val);
                    bytes.push((val, op.source.clone()));
                }

                nodes.push(AsmNode {
                    value: AsmNodeValue::Instruction(AsmInstruction::DefineBytes { bytes }),
                    source: main_operator.source.clone()
                });
            },

            PseudoInstructions::DefineString => {
                check_arg_count!(1);

                let op = &operands[0];
                let static_id = match op.value {
                    AsmValue::StringLiteral(id) => id,

                    _ => errors::parsing_error(&op.source, source, "Expected a string literal.")
                };

                nodes.push(AsmNode {
                    value: AsmNodeValue::Instruction(AsmInstruction::DefineString { string: static_id }),
                    source: main_operator.source.clone()
                });
            },
        },

        TokenValue::Bang => unreachable!("Handled before the match statement."),
        
        TokenValue::Number(_) |
        TokenValue::Identifier(_) |
        TokenValue::Dollar |
        TokenValue::Plus |
        TokenValue::Minus |
        TokenValue::Star |
        TokenValue::Div |
        TokenValue::StringLiteral(_) |
        TokenValue::CharLiteral(_) |
        TokenValue::Colon |
        TokenValue::EndMacro
            => errors::parsing_error(&main_operator.source, source, "Token cannot be used as a main operator.")
    }
    
}


pub fn parse<'a>(token_lines: &'a [TokenList<'a>], source: SourceCode, symbol_table: &'a SymbolTable<'a>) -> Vec<AsmNode<'a>> {

    // A good estimate for the number of nodes is the number of assembly lines. This is because an assembly line 
    // usually translates to a single instruction. This should avoid reallocations in most cases.
    let mut nodes = Vec::with_capacity(token_lines.len());

    let mut macros = MacroMap::new();

    let mut i: usize = 0;


    while let Some(line) = token_lines.get(i) {

        // Assume the line is not empty since the lexer has already filtered out empty lines

        let main_operator = &line[0];

        let operands = parse_operands(&line[1..], symbol_table, source);

        parse_line(main_operator, operands, &mut nodes, &mut macros, token_lines, &mut i, source, symbol_table);

        // Next line
        i += 1;
    }

    nodes.shrink_to_fit();
    nodes
}

