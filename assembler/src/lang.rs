use std::rc::Rc;

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


type AddressOperand<'a> = (AddressLike, Rc<SourceToken<'a>>);
type NumberOperand<'a> = (NumberLike, Rc<SourceToken<'a>>);


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

    LoadStatic1 { addr: AddressOperand<'a> },
    LoadStatic2 { addr: AddressOperand<'a> },
    LoadStatic4 { addr: AddressOperand<'a> },
    LoadStatic8 { addr: AddressOperand<'a> },
    LoadStaticBytes { addr: AddressOperand<'a>, count: NumberOperand<'a> },

    LoadConst1 { value: NumberOperand<'a> },
    LoadConst2 { value: NumberOperand<'a> },
    LoadConst4 { value: NumberOperand<'a> },
    LoadConst8 { value: NumberOperand<'a> },
    LoadConstBytes { bytes: Vec<NumberOperand<'a>> },

    Load1,
    Load2,
    Load4,
    Load8,
    LoadBytes,

    VirtualConstToReal { addr: AddressOperand<'a> },
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
    IntrConst { value: NumberOperand<'a> },

    ReadError,
    SetErrorConst { value: NumberOperand<'a> },
    SetError,

    Exit,

    JumpConst { addr: AddressOperand<'a> },
    Jump,
    JumpNotZeroConst1 { addr: AddressOperand<'a> },
    JumpNotZeroConst2 { addr: AddressOperand<'a> },
    JumpNotZeroConst4 { addr: AddressOperand<'a> },
    JumpNotZeroConst8 { addr: AddressOperand<'a> },
    JumpNotZero1,
    JumpNotZero2,
    JumpNotZero4,
    JumpNotZero8,
    JumpZeroConst1 { addr: AddressOperand<'a> },
    JumpZeroConst2 { addr: AddressOperand<'a> },
    JumpZeroConst4 { addr: AddressOperand<'a> },
    JumpZeroConst8 { addr: AddressOperand<'a> },
    JumpZero1,
    JumpZero2,
    JumpZero4,
    JumpZero8,
    JumpErrorConst { addr: AddressOperand<'a> },
    JumpError,
    JumpNoErrorConst { addr: AddressOperand<'a> },
    JumpNoError,

    DefineNumber { size: NumberOperand<'a>, value: NumberOperand<'a> },
    DefineBytes { bytes: Vec<NumberOperand<'a>> },
    DefineString { static_id: StaticID },

    Call { addr: AddressOperand<'a> },
    Return,

    Nop

}

// const_assert_eq!(mem::variant_count::<AsmInstruction>(), mem::variant_count::<ByteCodes>() + mem::variant_count::<PseudoInstructions>());


macro_rules! declare_pseudo_instructions {
    ($($name:ident $asm_name:ident),+) => {
        
/// Pseudo-instructions are assembly-only instructions that get evaluated at compile-time and have effects on the generated output byte code.
/// Each instruction is represented by one byte.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PseudoInstructions {
    $($name),+
}

impl PseudoInstructions {

    pub fn from_string(string: &str) -> Option<Self> {
        match string {
            $(stringify!($asm_name) => Some(Self::$name),)+
            _ => None
        }
    }

}

    };
}

declare_pseudo_instructions! {

    DefineNumber dn,
    DefineBytes db,
    DefineString ds,
    IncludeAsm include,
    Return ret

}


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
    MacroParameter(SymbolID),
}

impl AsmValue {

    fn as_uint_strict(&self) -> Option<u64> {
        match self {
            AsmValue::Const(n) => n.as_uint(),
            _ => None
        }
    }


    pub fn as_uint<'a>(&self, symbol_table: &'a SymbolTable<'a>) -> Option<u64> {
        match self {

            AsmValue::Const(n) => n.as_uint(),

            AsmValue::Symbol(id)
             => symbol_table.get_symbol(*id)
                .value
                .as_ref()
                .and_then(|v| v.as_uint_strict()),

            _ => None
        }
    }

}

