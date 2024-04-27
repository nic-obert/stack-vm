use std::rc::Rc;
use std::path::Path;
use std::borrow::Cow;

use crate::symbol_table::{StaticID, StaticValue, Symbol, SymbolID, SymbolTable};
use crate::lang::Number;
use crate::errors;

use hivmlib::ByteCodes;

use lazy_static::lazy_static;
use regex::Regex;


pub type TokenList<'a> = Vec<Token<'a>>;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(
        r#"(?m)'(?:\\'|[^'])*'|"(?:\\"|[^"])*"|[_a-zA-Z]\w*|0x[a-fA-F\d]+|-?\d+[.]\d*|-?[.]?\d+|[-+\/%@#$:.]|\S"#
    ).unwrap();

    static ref IDENTIFIER_REGEX: Regex = Regex::new(
        r#"[_a-zA-Z][_a-zA-Z\d]*"#
    ).unwrap();

}


#[derive(Debug)]
pub struct SourceToken<'a> {
    pub string: &'a str,
    pub unit_path: &'a Path,
    pub line_index: usize,
    pub column: usize
}

impl SourceToken<'_> {

    pub fn line_number(&self) -> usize {
        self.line_index + 1
    }

}


pub type SourceCode<'a> = &'a [&'a str];


fn escape_string_copy(string: &str, checked_until: usize, token: &SourceToken, source: SourceCode) -> String {
    // use -1 because the escape character won't be copied
    let mut s = String::with_capacity(string.len() - 1);
    
    // Copy the part of the string before the escape character
    s.push_str(&string[..checked_until]);

    let mut escape = true;

    for (i, c) in string[checked_until + 1..].chars().enumerate() {
        if escape {
            escape = false;
            s.push(match c { // Characters that are part of an escape sequence
                'n' => '\n',
                'r' => '\r',
                '0' => '\0',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                c => errors::invalid_escape_sequence(token, c, token.column + checked_until + i + 2, source)
            })
        } else if c == '\\' {
            escape = true;
        } else {
            s.push(c);
        }
    }

    s
}


fn escape_string<'a>(string: &'a str, token: &SourceToken, source: SourceCode) -> Cow<'a, str> {
    // Ignore the enclosing quote characters
    let string = &string[1..string.len() - 1];
    
    for (i, c) in string.chars().enumerate() {
        if c == '\\' {
            let copied_string = escape_string_copy(string, i, token, source);
            return Cow::Owned(copied_string);
        }
    }

    Cow::Borrowed(string)
}



fn lex<'a>(source: SourceCode<'a>, unit_path: &'a Path) -> impl Iterator<Item = Vec<SourceToken<'a>>> {

    source.iter().enumerate().filter_map(
        |(line_index, line)| {

            if line.trim().is_empty() {
                return None;
            }

            let mut matches = Vec::new();

            for mat in TOKEN_REGEX.find_iter(line) {

                match mat.as_str() {
                    ";" => break, // Ignore comments
                    s => matches.push(
                        SourceToken {
                            string: s,
                            unit_path,
                            line_index,
                            column: mat.start() + 1
                        }
                    )
                }
            }

            if matches.is_empty() {
                None
            } else {
                Some(matches)
            }
        }
    )
}


#[derive(Debug)]
pub struct Token<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub value: TokenValue,
    priority: TokenPriority
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}


#[derive(Debug)]
pub enum TokenValue {
    StringLiteral(StaticID),
    CharLiteral(char),
    Colon,
    At,
    Number(Number),
    Identifier(SymbolID),
    Instruction(ByteCodes),
    Dollar,
    Plus,
    Minus,
    Star,
    Div,
    Mod,
    Dot,
}

impl TokenValue {

    fn base_priority(&self) -> TokenBasePriority {
        match self {
            TokenValue::StringLiteral(_) |
            TokenValue::CharLiteral(_) |
            TokenValue::Number(_) | 
            TokenValue::Identifier(_) |
            TokenValue::Colon 
                => TokenBasePriority::None,

            TokenValue::Instruction(_) => TokenBasePriority::Instruction,

            TokenValue::Plus |
            TokenValue::Minus
                => TokenBasePriority::PlusMinus,

            TokenValue::Star |
            TokenValue::Div |
            TokenValue::Mod
                => TokenBasePriority::MulDivMod,

            TokenValue::Dollar |
            TokenValue::At |
            TokenValue::Dot
                => TokenBasePriority::AsmOperator,
        }
    }

}


#[derive(PartialOrd, PartialEq)]
enum TokenBasePriority {
    None = 0,
    Instruction,
    PlusMinus,
    MulDivMod,
    AsmOperator,
}


/// Total priority of a token. u8 should be enough for assembly.
type TokenPriority = u8;


#[inline]
fn is_decimal_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


pub fn tokenize<'a>(source: SourceCode<'a>, unit_path: &'a Path, symbol_table: &mut SymbolTable<'a>) -> Vec<TokenList<'a>> {
    
    let raw_lines = lex(source, unit_path);

    let mut lines = Vec::with_capacity(source.len());

    for raw_line in raw_lines {

        let mut current_line = TokenList::new();

        for token in raw_line {

            let token_rc = Rc::new(token);
            let token = token_rc.as_ref();

            let token_value = match token_rc.string {

                "." => TokenValue::Dot,
                
                "+" => TokenValue::Plus,
                
                "-" => TokenValue::Minus,
                
                "*" => TokenValue::Star,
                
                "/" => TokenValue::Div,

                "%" => TokenValue::Mod,

                ":" => TokenValue::Colon,

                "$" => TokenValue::Dollar,

                "@" => TokenValue::At,

                string if string.starts_with("0x") => {
                    TokenValue::Number(Number::Uint(u64::from_str_radix(&string[2..], 16).unwrap_or_else(|err| errors::parsing_error(token, source, err.to_string().as_str()))))
                },

                string if string.starts_with(is_decimal_numeric) => {
                    TokenValue::Number(if string.contains('.') {
                        Number::Float(string.parse::<f64>().unwrap_or_else(|err| errors::parsing_error(token, source, err.to_string().as_str())))
                    } else if string.starts_with('-') {
                        Number::Int(string.parse::<i64>().unwrap_or_else(|err| errors::parsing_error(token, source, err.to_string().as_str())))
                    } else {
                        Number::Uint(string.parse::<u64>().unwrap_or_else(|err| errors::parsing_error(token, source, err.to_string().as_str())))
                    })
                },

                string if string.starts_with('"') => {

                    if !string.ends_with('"') {
                        errors::parsing_error(token, source, "Unterminated string literal.");
                    }

                    let string = escape_string(string, token, source);
                    let static_id = symbol_table.declare_static(StaticValue::StringLiteral(string));

                    TokenValue::StringLiteral(static_id)
                },
        
                string if string.starts_with('\'') => {

                    if !string.ends_with('\'') {
                        errors::parsing_error(token, source, "Unterminated character literal.");
                    }

                    let escaped_string = escape_string(string, token, source);

                    if escaped_string.len() != 3 {
                        errors::parsing_error(token, source, "Invalid character literal. A character literal can only contain one character.");
                    }

                    TokenValue::CharLiteral(
                        escaped_string.chars().next().unwrap()
                    )
                },

                string => {

                    if let Some(instruction) = ByteCodes::from_string(string) {
                        TokenValue::Instruction(instruction)
                        
                    } else if IDENTIFIER_REGEX.is_match(string) {

                        if let Some(symbol_id) = symbol_table.get_symbol_id(string) {
                            TokenValue::Identifier(symbol_id)
                        } else {
                            let symbol_id = match symbol_table.declare_symbol(string, Symbol { source: token_rc.clone(), value: None, name: string }) {
                                Ok(id) => id,
                                Err(old_symbol) => errors::symbol_redeclaration(token, source, &old_symbol.borrow())
                            };
    
                            TokenValue::Identifier(symbol_id)
                        }
                        
                    } else {
                        errors::parsing_error(token, source, "Invalid token.")
                    }
                }

            };

            current_line.push(Token {
                source: token_rc,
                priority: token_value.base_priority() as TokenPriority,
                value: token_value,
            });

        }

        lines.push(current_line);
    }

    lines
}

