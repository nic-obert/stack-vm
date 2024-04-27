use std::{mem, rc::Rc};

use static_assertions::const_assert_eq;

use hivmlib::ByteCodes;

use crate::{symbol_table::{StaticID, SymbolID}, tokenizer::SourceToken};


#[derive(Debug, Clone)]
pub enum Number {
    Uint(u64),
    Int(i64),
    Float(f64)    
}


#[derive(Debug)]
pub enum NumberLike {
    Number(Number),
    Symbol(SymbolID),
    CurrentPosition,
}

pub type AddressLike = NumberLike;

/// Representation of assembly instructions and their operands
#[derive(Debug)]
pub enum AsmInstruction {
    
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

    LoadStatic1 { addr: AddressLike },
    LoadStatic2 { addr: AddressLike },
    LoadStatic4 { addr: AddressLike },
    LoadStatic8 { addr: AddressLike },
    LoadStaticBytes { addr: AddressLike },

    LoadConst1 { value: NumberLike },
    LoadConst2 { value: NumberLike },
    LoadConst4 { value: NumberLike },
    LoadConst8 { value: NumberLike },
    LoadConstBytes { bytes: Vec<NumberLike> },

    Load1,
    Load2,
    Load4,
    Load8,
    LoadBytes,

    VirtualConstToReal { addr: AddressLike },
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
    IntrConst { code: NumberLike },

    Exit,

    JumpConst { addr: AddressLike },
    Jump,
    JumpNotZeroConst1 { addr: AddressLike },
    JumpNotZeroConst2 { addr: AddressLike },
    JumpNotZeroConst4 { addr: AddressLike },
    JumpNotZeroConst8 { addr: AddressLike },
    JumpNotZero1,
    JumpNotZero2,
    JumpNotZero4,
    JumpNotZero8,
    JumpZeroConst1 { addr: AddressLike },
    JumpZeroConst2 { addr: AddressLike },
    JumpZeroConst4 { addr: AddressLike },
    JumpZeroConst8 { addr: AddressLike },
    JumpZero1,
    JumpZero2,
    JumpZero4,
    JumpZero8,

    Nop

}

const_assert_eq!(mem::variant_count::<AsmInstruction>(), mem::variant_count::<ByteCodes>());


#[derive(Debug)]
pub enum AsmSection<'a> {
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


    pub fn name(&self) -> &'a str {
        match self {
            AsmSection::Text => "text",
            AsmSection::Data => "data",
            AsmSection::Generic(name) => name
        }
    }

}


#[derive(Debug)]
pub struct AsmNode<'a> {

    pub value: AsmNodeValue<'a>,
    pub source: Rc<SourceToken<'a>>

}


#[derive(Debug)]
pub enum AsmNodeValue<'a> {
    Instruction(AsmInstruction),
    Label(&'a str),
    Section(AsmSection<'a>),
    // MacroDef, TODO
    // MacroCall, TODO

}


#[derive(Debug, Clone)]
pub enum AsmValue {
    Const(Number),
    CurrentPosition,
    StringLiteral(StaticID),
    Symbol(SymbolID),
}

