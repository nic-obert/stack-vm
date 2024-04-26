use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::{collections::HashMap, fmt::Debug};
use std::path::Path;
use std::borrow::Cow;
use std::mem;

use static_assertions::const_assert_eq;
use lazy_static::lazy_static;
use regex::Regex;

use hivmlib::{ByteCodes, Interrupts, VirtualAddress};

use crate::errors;


type TokenList<'a> = Vec<Token<'a>>;


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

            Some(matches)
        }
    )
}



#[derive(Debug)]
struct Token<'a> {
    source: Rc<SourceToken<'a>>,
    value: TokenValue,
    priority: TokenPriority
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}


#[derive(Debug, Clone)]
enum Number {
    Uint(u64),
    Int(i64),
    Float(f64)    
}


#[derive(Debug)]
enum TokenValue {
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


fn tokenize<'a>(source: SourceCode<'a>, unit_path: &'a Path, symbol_table: &mut SymbolTable<'a>) -> Vec<TokenList<'a>> {
    
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

                        let symbol_id = match symbol_table.declare_symbol(string, Symbol { source: token_rc.clone(), value: None}) {
                            Ok(id) => id,
                            Err(old_symbol) => errors::parsing_error(token, source, "Symbol already declared in the current scope.") // TODO: print the location of the previous declaration
                        };

                        TokenValue::Identifier(symbol_id)
                    } else {
                        errors::parsing_error(token, source, "Invalid token.")
                    }
                }

            };

            current_line.push(Token {
                source: token_rc,
                priority: token_value.base_priority() as TokenPriority, // TODO: implement increased priority inside delimiters
                value: token_value,
            });

        }

        lines.push(current_line);
    }

    lines
}


struct Symbol<'a> {

    source: Rc<SourceToken<'a>>,
    value: Option<AsmValue>,

}


struct Scope<'a> {
    symbols: HashMap<&'a str, SymbolID>,
}

impl Scope<'_> {

    pub fn new() -> Self {
        Self {
            symbols: HashMap::new()
        }
    }

}


#[derive(Debug, Clone, Copy)]
struct SymbolID(usize);


enum StaticValue<'a> {
    StringLiteral(Cow<'a, str>)
}


struct SymbolTable<'a> {

    scopes: Vec<Scope<'a>>,
    symbols: Vec<RefCell<Symbol<'a>>>,
    statics: Vec<StaticValue<'a>>

}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()], // Start with the global scope already pushed
            symbols: Vec::new(),
            statics: Vec::new()
        }
    }


    pub fn declare_static(&mut self, value: StaticValue<'a>) -> StaticID {
        let id = self.statics.len();
        self.statics.push(value);
        StaticID(id)
    }

    
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }


    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }


    pub fn define_symbol(&self, id: SymbolID, value: AsmValue) {
        self.symbols[id.0].borrow_mut().value = Some(value);
    }


    /// Returns None if the symbol is already declared in the current scope.
    pub fn declare_symbol(&mut self, name: &'a str, symbol: Symbol<'a>) -> Result<SymbolID, &RefCell<Symbol<'a>>> {

        let scope = self.scopes.last_mut().unwrap();

        let symbol_id = SymbolID(self.symbols.len());
        if let Some(old_symbol) = scope.symbols.insert(name, symbol_id.clone()) {
            return Err(&self.symbols[old_symbol.0]);
        }

        self.symbols.push(RefCell::new(symbol));
        Ok(symbol_id)
    }


    pub fn get_symbol_id(&self, name: &str) -> Option<SymbolID> {
        self.scopes.iter().rev().find_map(|scope| scope.symbols.get(name).cloned())
    }


    pub fn get_symbol(&self, id: SymbolID) -> Option<&RefCell<Symbol<'a>>> {
        self.symbols.get(id.0)
    }

    // pub fn get_symbol_value(&self, name: &str) -> Option<AsmValue<'a>> {
    //     self.get_symbol(name).map(|symbol| symbol.borrow().value.clone())?
    // }

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


enum AsmSection<'a> {
    Text,
    Data,
    Generic(&'a str)
}

impl<'a> AsmSection<'a> {

    pub fn from_name(name: &'a str) -> AsmSection<'a> {
        match name {
            "text" => AsmSection::Text,
            "data" => AsmSection::Data,
            _ => AsmSection::Generic(name)
        }
    }

}


enum AsmNode<'a> {
    Instruction(AsmInstruction),
    Label(&'a str),
    Section(AsmSection<'a>),
    // MacroDef, TODO
    // MacroCall, TODO

}


#[derive(Debug, Clone, Copy)]
struct StaticID(usize);


#[derive(Debug, Clone)]
enum AsmValue {
    Const(Number),
    CurrentPosition,
    StringLiteral(StaticID),
    Symbol(SymbolID),
}


fn get_highest_priority(tokens: &TokenList) -> Option<usize> {
    
    let mut highest_priority = TokenBasePriority::None as TokenPriority;
    let mut highest_priority_index = None;

    for (index, token) in tokens.iter().enumerate() {
        if token.priority > highest_priority {
            highest_priority = token.priority;
            highest_priority_index = Some(index);
        }
    }

    highest_priority_index
}


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


fn parse<'a>(token_lines: &'a [TokenList<'a>], source: SourceCode, symbol_table: &'a mut SymbolTable<'a>) -> Vec<AsmNode<'a>> {

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

                symbol_table.define_symbol(symbol_id, AsmValue::Symbol(symbol_id));

                let label_name = symbol_table.get_symbol(symbol_id).unwrap().borrow().source.string;

                nodes.push(AsmNode::Label(label_name));
            },

            TokenValue::Instruction(code) => {
                match code {
                    ByteCodes::AddInt1 => todo!(),
                    ByteCodes::AddInt2 => todo!(),
                    ByteCodes::AddInt4 => todo!(),
                    ByteCodes::AddInt8 => todo!(),
                    ByteCodes::SubInt1 => todo!(),
                    ByteCodes::SubInt2 => todo!(),
                    ByteCodes::SubInt4 => todo!(),
                    ByteCodes::SubInt8 => todo!(),
                    ByteCodes::MulInt1 => todo!(),
                    ByteCodes::MulInt2 => todo!(),
                    ByteCodes::MulInt4 => todo!(),
                    ByteCodes::MulInt8 => todo!(),
                    ByteCodes::DivInt1 => todo!(),
                    ByteCodes::DivInt2 => todo!(),
                    ByteCodes::DivInt4 => todo!(),
                    ByteCodes::DivInt8 => todo!(),
                    ByteCodes::ModInt1 => todo!(),
                    ByteCodes::ModInt2 => todo!(),
                    ByteCodes::ModInt4 => todo!(),
                    ByteCodes::ModInt8 => todo!(),
                    ByteCodes::AddFloat4 => todo!(),
                    ByteCodes::AddFloat8 => todo!(),
                    ByteCodes::SubFloat4 => todo!(),
                    ByteCodes::SubFloat8 => todo!(),
                    ByteCodes::MulFloat4 => todo!(),
                    ByteCodes::MulFloat8 => todo!(),
                    ByteCodes::DivFloat4 => todo!(),
                    ByteCodes::DivFloat8 => todo!(),
                    ByteCodes::ModFloat4 => todo!(),
                    ByteCodes::ModFloat8 => todo!(),
                    ByteCodes::LoadStatic1 => todo!(),
                    ByteCodes::LoadStatic2 => todo!(),
                    ByteCodes::LoadStatic4 => todo!(),
                    ByteCodes::LoadStatic8 => todo!(),
                    ByteCodes::LoadStaticBytes => todo!(),
                    ByteCodes::LoadConst1 => todo!(),
                    ByteCodes::LoadConst2 => todo!(),
                    ByteCodes::LoadConst4 => todo!(),
                    ByteCodes::LoadConst8 => todo!(),
                    ByteCodes::LoadConstBytes => todo!(),
                    ByteCodes::Load1 => todo!(),
                    ByteCodes::Load2 => todo!(),
                    ByteCodes::Load4 => todo!(),
                    ByteCodes::Load8 => todo!(),
                    ByteCodes::LoadBytes => todo!(),
                    ByteCodes::VirtualConstToReal => todo!(),
                    ByteCodes::VirtualToReal => todo!(),
                    ByteCodes::Store1 => todo!(),
                    ByteCodes::Store2 => todo!(),
                    ByteCodes::Store4 => todo!(),
                    ByteCodes::Store8 => todo!(),
                    ByteCodes::StoreBytes => todo!(),
                    ByteCodes::Memmove1 => todo!(),
                    ByteCodes::Memmove2 => todo!(),
                    ByteCodes::Memmove4 => todo!(),
                    ByteCodes::Memmove8 => todo!(),
                    ByteCodes::MemmoveBytes => todo!(),
                    ByteCodes::Duplicate1 => todo!(),
                    ByteCodes::Duplicate2 => todo!(),
                    ByteCodes::Duplicate4 => todo!(),
                    ByteCodes::Duplicate8 => todo!(),
                    ByteCodes::DuplicateBytes => todo!(),
                    ByteCodes::Malloc => todo!(),
                    ByteCodes::Realloc => todo!(),
                    ByteCodes::Free => todo!(),
                    ByteCodes::Intr => todo!(),
                    ByteCodes::IntrConst => todo!(),
                    ByteCodes::Exit => todo!(),
                    ByteCodes::Nop => todo!(),
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

                symbol_table.define_symbol(symbol_id, AsmValue::Symbol(symbol_id));

                let section_name = symbol_table.get_symbol(symbol_id).unwrap().borrow().source.string;

                nodes.push(AsmNode::Section(AsmSection::from_name(section_name)));                
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


pub fn assemble<'a>(raw_source: &'a str, unit_path: &'a Path) -> Vec<u8> {

    let source_lines = raw_source.lines().collect::<Vec<_>>();
    
    let mut symbol_table = SymbolTable::new();

    let token_lines = tokenize(&source_lines, unit_path, &mut symbol_table);

    for line in &token_lines {
        for token in line.iter() {
            println!("{}", token);
        }
    }


    let asm = parse(&token_lines, &source_lines, &mut symbol_table);

    todo!()
}

