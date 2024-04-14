use std::collections::HashMap;
use std::path::Path;
use std::borrow::Cow;
use std::mem;

use static_assertions::const_assert_eq;
use lazy_static::lazy_static;
use regex::Regex;

use hivmlib::{ByteCodes, Interrupts, VirtualAddress};

use crate::errors;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(
        r#"(?m)'(?:\\'|[^'])*'|"(?:\\"|[^"])*"|[_a-zA-Z]\w*|0x[a-fA-F\d]+|-?\d+[.]\d*|-?[.]?\d+|[-+\/%@#$:.=]|\S"#
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



fn lex<'a>(source: SourceCode<'a>, unit_path: &'a Path) -> impl Iterator<Item = SourceToken<'a>> {

    source.iter().enumerate().flat_map(
        |(line_index, line)| {

            if line.trim().is_empty() {
                return Vec::new();
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

            // Add a final newline token that will be used as a delimiter during parsing
            matches.push(
                SourceToken {
                    string: "\n",
                    unit_path,
                    line_index,
                    column: line.len()
                }
            );

            matches
        }
    )
}



#[derive(Debug)]
struct Token<'a> {
    source: SourceToken<'a>,
    value: TokenValue<'a>
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}


#[derive(Debug)]
enum Number {
    Uint(u64),
    Int(i64),
    Float(f64)    
}


#[derive(Debug)]
enum TokenValue<'a> {
    StringLiteral(Cow<'a, str>),
    CharLiteral(char),
    Colon,
    At,
    Number(Number),
    Identifier(&'a str),
    Instruction(ByteCodes),
    Dollar,
    Plus,
    Minus,
    Star,
    Div,
    Mod,
    Dot,
    Equal,
    Endline,
}

impl TokenValue<'_> {

    fn base_priority(&self) -> TokenPriority {
        match self {
            TokenValue::StringLiteral(_) |
            TokenValue::CharLiteral(_) |
            TokenValue::Number(_) | 
            TokenValue::Identifier(_) |
            TokenValue::Colon |
            TokenValue::Endline
                => TokenPriority::None,

            TokenValue::Instruction(_) => TokenPriority::Instruction,

            TokenValue::Plus |
            TokenValue::Minus
                => TokenPriority::PlusMinus,

            TokenValue::Star |
            TokenValue::Div |
            TokenValue::Mod
                => TokenPriority::MulDivMod,

            TokenValue::Dollar |
            TokenValue::At |
            TokenValue::Dot |
            TokenValue::Equal
                => TokenPriority::AsmOperator,
        }
    }

}


#[derive(PartialOrd, PartialEq)]
enum TokenPriority {
    None = 0,
    Instruction,
    PlusMinus,
    MulDivMod,
    AsmOperator,
    Max, // TODO: maybe this shouldn't exist??
}


#[inline]
fn is_decimal_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


fn tokenize<'a>(source: SourceCode<'a>, unit_path: &'a Path) -> Vec<Token<'a>> {
    
    let raw_tokens = lex(source, unit_path);

    let mut tokens = Vec::new();

    for token in raw_tokens {

        let token_value = match token.string {

            "\n" => TokenValue::Endline,

            "=" => TokenValue::Equal,

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
                TokenValue::Number(Number::Uint(u64::from_str_radix(&string[2..], 16).unwrap_or_else(|err| errors::parsing_error(&token, source, err.to_string().as_str()))))
            },

            string if string.starts_with(is_decimal_numeric) => {
                TokenValue::Number(if string.contains('.') {
                    Number::Float(string.parse::<f64>().unwrap_or_else(|err| errors::parsing_error(&token, source, err.to_string().as_str())))
                } else if string.starts_with('-') {
                    Number::Int(string.parse::<i64>().unwrap_or_else(|err| errors::parsing_error(&token, source, err.to_string().as_str())))
                } else {
                    Number::Uint(string.parse::<u64>().unwrap_or_else(|err| errors::parsing_error(&token, source, err.to_string().as_str())))
                })
            },

            string if string.starts_with('"') => {

                if !string.ends_with('"') {
                    errors::parsing_error(&token, source, "Unterminated string literal.");
                }

                TokenValue::StringLiteral(escape_string(string, &token, source))
            },
    
            string if string.starts_with('\'') => {

                if !string.ends_with('\'') {
                    errors::parsing_error(&token, source, "Unterminated character literal.");
                }

                let escaped_string = escape_string(string, &token, source);

                if escaped_string.len() != 3 {
                    errors::parsing_error(&token, source, "Invalid character literal. A character literal can only contain one character.");
                }

                TokenValue::CharLiteral(
                    escaped_string.chars().next().unwrap()
                )
            },

            string => {

                if let Some(instruction) = ByteCodes::from_string(string) {
                    TokenValue::Instruction(instruction)
                } else if IDENTIFIER_REGEX.is_match(string) {
                    TokenValue::Identifier(string)
                } else {
                    errors::parsing_error(&token, source, "Invalid token.")
                }
            }

        };

        tokens.push(Token {
            source: token,
            value: token_value
        });

    }

    tokens
}


struct Symbol<'a> {

    source: SourceToken<'a>,

}


struct SymbolTable<'a> {

    symbols: HashMap<&'a str, Symbol<'a>>

}


/// Representation of assembly instructions and their operands
enum AsmInstruction {
    
    AddInt1,
    AddInt2,
    AddInt4,
    AddInt8,
    SubInt1,
    SubInt2,
    SubInt4,
    SubInt8,
    MulInt1,
    MulInt2,
    MulInt4,
    MulInt8,
    DivInt1,
    DivInt2,
    DivInt4,
    DivInt8,
    ModInt1,
    ModInt2,
    ModInt4,
    ModInt8,

    AddFloat4,
    AddFloat8,
    SubFloat4,
    SubFloat8,
    MulFloat4,
    MulFloat8,
    DivFloat4,
    DivFloat8,
    ModFloat4,
    ModFloat8,

    LoadStatic1 { addr: VirtualAddress },
    LoadStatic2 { addr: VirtualAddress },
    LoadStatic4 { addr: VirtualAddress },
    LoadStatic8 { addr: VirtualAddress },
    LoadStaticBytes { addr: VirtualAddress },

    LoadConst1 { value: u8 },
    LoadConst2 { value: u16 },
    LoadConst4 { value: u32 },
    LoadConst8 { value: u64 },
    LoadConstBytes { value: Vec<u8> }, // TODO: maybe this should be a slice?

    Load1,
    Load2,
    Load4,
    Load8,
    LoadBytes,

    VirtualConstToReal { addr: VirtualAddress },
    VirtualToReal,

    Store1,
    Store2,
    Store4,
    Store8,
    StoreBytes,

    Memmove1,
    Memmove2,
    Memmove4,
    Memmove8,
    MemmoveBytes,

    Duplicate1,
    Duplicate2,
    Duplicate4,
    Duplicate8,
    DuplicateBytes,

    Malloc,
    Realloc,
    Free,

    Intr,
    IntrConst { code: Interrupts },

    Exit,

    Nop

}

const_assert_eq!(mem::variant_count::<AsmInstruction>(), mem::variant_count::<ByteCodes>());


enum AsmSection {
    Text,
    Static,
}


enum AsmNode<'a> {
    Instruction(AsmInstruction),
    Label(&'a str),
    Section(AsmSection),
    // MacroDef, TODO
    // MacroCall, TODO

}


fn get_highest_priority<'a>(tokens: &'a [Token<'a>]) -> Option<usize> {
    
    let mut highest_priority = TokenPriority::None;
    let mut highest_priority_index = None;

    for (i, token) in tokens.iter().enumerate() {

        if matches!(token.value, TokenValue::Endline) {
            break;
        }

        let priority = token.value.base_priority();
        if priority > highest_priority {
            highest_priority = priority;
            highest_priority_index = Some(i);
        }
    }

    highest_priority_index
}


fn next_line<'a>(tokens: &'a [Token<'a>]) -> Option<&'a [Token<'a>]> {
    let mut i = 0;
    while i < tokens.len() && !matches!(tokens[i].value, TokenValue::Endline) {
        i += 1;
    }

    if i == tokens.len() {
        None
    } else {
        Some(&tokens[i + 1..])
    }
}


fn parse<'a>(mut tokens: &'a [Token<'a>], source: SourceCode<'a>, unit_path: &'a Path) -> Vec<AsmNode<'a>> {
    
    let mut nodes = Vec::new();

    while let Some(line) = next_line(tokens) {

        tokens = &tokens[line.len()..];

        let highest_priority_index = get_highest_priority(line);

        // TODO: maybe we need to use a linked list for the tokens?
        // could be overkill, though. 

    }

    nodes
}


pub fn assemble<'a>(raw_source: &'a str, unit_path: &'a Path) -> Vec<u8> {

    let source = raw_source.lines().collect::<Vec<_>>();
    
    let tokens = tokenize(&source, unit_path);

    for token in &tokens {
        println!("{}", token);
    }

    let asm = parse(&tokens, &source, unit_path);

    todo!()
}

