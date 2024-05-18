use std::rc::Rc;
use std::mem;

use static_assertions::const_assert_eq;

use hivmlib::ByteCodes;

use crate::tokenizer::SourceToken;
use crate::symbol_table::{StaticID, SymbolID, SymbolTable};


pub const ENTRY_SECTION_NAME: &str = "text";


#[derive(Debug, Clone)]
pub enum Number {
    Uint(u64),
    Int(i64),
    Float(f64)    
}

impl Number {

    pub fn minimum_size(&self) -> u8 {
        // A bit ugly, but it works for this purpose
        match self {
            Number::Uint(value) => {
                if *value <= u8::MAX as u64 {
                    1
                } else if *value <= u16::MAX as u64 {
                    2
                } else if *value <= u32::MAX as u64 {
                    4
                } else {
                    8
                }
            },
            Number::Int(value) => {
                if *value >= i8::MIN as i64 && *value <= i8::MAX as i64 {
                    1
                } else if *value >= i16::MIN as i64 && *value <= i16::MAX as i64 {
                    2
                } else if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 {
                    4
                } else {
                    8
                }
            },
            Number::Float(value) => {
                if *value >= f32::MIN as f64 && *value <= f32::MAX as f64 {
                    4
                } else {
                    8
                }
            }
        }
    }

    pub fn as_le_bytes(&self) -> Vec<u8> {
        match self {
            Number::Uint(value) => value.to_le_bytes().to_vec(),
            Number::Int(value) => value.to_le_bytes().to_vec(),
            Number::Float(value) => value.to_le_bytes().to_vec()
        }
    }


    pub fn as_uint(&self) -> Option<u64> {
        match self {
            Number::Uint(value) => Some(*value),
            _ => None
        }
    }

}


#[derive(Debug)]
pub enum NumberLike {
    Number(Number, u8),
    Symbol(SymbolID),
    CurrentPosition,
}

impl NumberLike {

    pub fn from_number(n: &Number) -> Self {
        NumberLike::Number(n.clone(), n.minimum_size())
    }

}


#[derive(Debug)]
pub enum AddressLike {
    Number(Number),
    Symbol(SymbolID),
    CurrentPosition,
}


/// Representation of assembly instructions and their operands
#[derive(Debug)]
pub enum AsmInstruction<'a> {
    
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

    LoadStatic1 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    LoadStatic2 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    LoadStatic4 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    LoadStatic8 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    LoadStaticBytes { addr: (AddressLike, Rc<SourceToken<'a>>), count: (NumberLike, Rc<SourceToken<'a>>) },

    LoadConst1 { value: (NumberLike, Rc<SourceToken<'a>>) },
    LoadConst2 { value: (NumberLike, Rc<SourceToken<'a>>) },
    LoadConst4 { value: (NumberLike, Rc<SourceToken<'a>>) },
    LoadConst8 { value: (NumberLike, Rc<SourceToken<'a>>) },
    LoadConstBytes { bytes: Vec<(NumberLike, Rc<SourceToken<'a>>)> },

    Load1,
    Load2,
    Load4,
    Load8,
    LoadBytes,

    VirtualConstToReal { addr: (AddressLike, Rc<SourceToken<'a>>) },
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
    IntrConst { code: (NumberLike, Rc<SourceToken<'a>>) },

    Exit,

    JumpConst { addr: (AddressLike, Rc<SourceToken<'a>>) },
    Jump,
    JumpNotZeroConst1 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpNotZeroConst2 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpNotZeroConst4 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpNotZeroConst8 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpNotZero1,
    JumpNotZero2,
    JumpNotZero4,
    JumpNotZero8,
    JumpZeroConst1 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpZeroConst2 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpZeroConst4 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpZeroConst8 { addr: (AddressLike, Rc<SourceToken<'a>>) },
    JumpZero1,
    JumpZero2,
    JumpZero4,
    JumpZero8,

    Nop

}

const_assert_eq!(mem::variant_count::<AsmInstruction>(), mem::variant_count::<ByteCodes>());


#[derive(Debug)]
pub struct AsmNode<'a> {

    pub value: AsmNodeValue<'a>,
    pub source: Rc<SourceToken<'a>>

}


#[derive(Debug, Clone)]
pub struct AsmOperand<'a> {

    pub value: AsmValue,
    pub source: Rc<SourceToken<'a>>

}


#[derive(Debug)]
pub enum AsmNodeValue<'a> {
    Instruction(AsmInstruction<'a>),
    Label(&'a str),
    Section(&'a str),
}


#[derive(Debug, Clone)]
pub enum AsmValue {
    Const(Number),
    CurrentPosition,
    StringLiteral(StaticID),
    Symbol(SymbolID),
    MacroSymbol(SymbolID),
}

impl AsmValue {

    fn as_uint_strict(&self) -> Option<u64> {
        match self {
            AsmValue::Const(n) => n.as_uint(),
            _ => None
        }
    }


    pub fn as_uint(&self, symbol_table: &SymbolTable) -> Option<u64> {
        match self {

            AsmValue::Const(n) => n.as_uint(),

            AsmValue::Symbol(id)
             => symbol_table.get_symbol(*id)
                .borrow()
                .value
                .as_ref()
                .and_then(|v| v.as_uint_strict()),

            _ => None
        }
    }

}

