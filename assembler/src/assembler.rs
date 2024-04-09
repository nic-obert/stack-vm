use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use hivmlib::ByteCodes;

use crate::errors;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(
        r#"(?m)((?:'|").*(?:'|"))|[_a-zA-Z][_A-Za-z\d]*|[+-]?\d+[.]\d*|[+-]?[.]\d+|0x[a-fA-F\d]+|[@#:]\S"#
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


fn lex<'a>(source: &'a str, unit_path: &'a Path) -> impl Iterator<Item = SourceToken<'a>> {

    source.lines().enumerate().flat_map(
        |(line_index, line)| {

            if line.trim().is_empty() {
                return Vec::new();
            }

            let mut matches = Vec::new();

            for mat in TOKEN_REGEX.find_iter(line) {

                match mat.as_str() {
                    "#" => break, // Ignore comments
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
            matches
        }
    )
}



#[derive(Debug)]
struct Token<'a> {
    source: SourceToken<'a>,
    value: TokenValue<'a>
}


#[derive(Debug)]
enum Number {
    Uint(u64),
    Int(i64),
    Float(f64)    
}


#[derive(Debug)]
enum TokenValue<'a> {
    StringLiteral(&'a str),
    CharLiteral(char),
    Colon,
    At,
    Number(Number),
    Identifier(&'a str),
    Instruction(ByteCodes),
}


#[inline]
fn is_decimal_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


fn tokenize<'a>(source: &'a str, unit_path: &'a Path) -> Vec<Token<'a>> {
    
    let raw_tokens = lex(source, unit_path);

    let mut tokens = Vec::new();

    for token in raw_tokens {

        let token_value = match token.string {

            ":" => TokenValue::Colon,

            "@" => TokenValue::At,

            string if string.starts_with("0x") => {
                TokenValue::Number(Number::Uint(string.parse::<u64>().unwrap_or_else(|err| errors::parsing_error(&token, err.to_string().as_str()))))
            },

            string if string.starts_with(is_decimal_numeric) => {
                TokenValue::Number(if string.contains('.') {
                    Number::Float(string.parse::<f64>().unwrap_or_else(|err| errors::parsing_error(&token, err.to_string().as_str())))
                } else if string.starts_with('-') {
                    Number::Int(string.parse::<i64>().unwrap_or_else(|err| errors::parsing_error(&token, err.to_string().as_str())))
                } else {
                    Number::Uint(string.parse::<u64>().unwrap_or_else(|err| errors::parsing_error(&token, err.to_string().as_str())))
                })
            },

            string if string.starts_with('"') => {
                TokenValue::StringLiteral(&string[1..string.len()-1])
            },
    
            string if string.starts_with('\'') => {
                TokenValue::CharLiteral(
                    string.chars().nth(1).unwrap() // TODO: Handle escape sequences and errors
                )
            },

            string => {

                if let Some(instruction) = ByteCodes::from_string(string) {
                    TokenValue::Instruction(instruction)
                } else if IDENTIFIER_REGEX.is_match(string) {
                    TokenValue::Identifier(string)
                } else {
                    errors::parsing_error(&token, "Invalid token")
                }
            }

        };

        tokens.push(Token {
            source: token,
            value: token_value
        });

    }

    todo!()
}


pub fn assemble<'a>(source: &'a str, unit_path: &'a Path) -> Vec<u8> {
    
    let tokens = tokenize(source, unit_path);

    todo!()
}

