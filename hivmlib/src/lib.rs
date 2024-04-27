use std::mem;

use static_assertions::const_assert_eq;


pub type Address = usize;

#[derive(Default)]
pub struct VirtualAddress(pub Address);

const_assert_eq!(mem::size_of::<VirtualAddress>(), mem::size_of::<usize>());


pub type ByteCode<'a> = &'a [u8];


/// HiVM instructions. Each instruction is represented by one byte.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ByteCodes {

    AddInt1 = 0,
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

    LoadStatic1,
    LoadStatic2,
    LoadStatic4,
    LoadStatic8,
    LoadStaticBytes,

    LoadConst1,
    LoadConst2,
    LoadConst4,
    LoadConst8,
    LoadConstBytes,

    Load1,
    Load2,
    Load4,
    Load8,
    LoadBytes,
    
    VirtualConstToReal,
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
    IntrConst,

    Exit,

    JumpConst,
    Jump,

    JumpNotZeroConst1,
    JumpNotZeroConst2,
    JumpNotZeroConst4,
    JumpNotZeroConst8,
    JumpNotZero1,
    JumpNotZero2,
    JumpNotZero4,
    JumpNotZero8,
    JumpZeroConst1,
    JumpZeroConst2,
    JumpZeroConst4,
    JumpZeroConst8,
    JumpZero1,
    JumpZero2,
    JumpZero4,
    JumpZero8,

    /// No operation. Do nothing for this cycle.
    Nop

}

impl From<u8> for ByteCodes {
    /// Convert a byte to an intruction code. An invalid interrupt code will result in undefined behavior.
    fn from(byte: u8) -> Self {
        unsafe {
            mem::transmute(byte)
        }
    }
}

impl ByteCodes {

    pub fn from_string(string: &str) -> Option<Self> {
        match string {
            "addi1" => Some(Self::AddInt1),
            "addi2" => Some(Self::AddInt2),
            "addi4" => Some(Self::AddInt4),
            "addi8" => Some(Self::AddInt8),
            "subi1" => Some(Self::SubInt1),
            "subi2" => Some(Self::SubInt2),
            "subi4" => Some(Self::SubInt4),
            "subi8" => Some(Self::SubInt8),
            "muli1" => Some(Self::MulInt1),
            "muli2" => Some(Self::MulInt2),
            "muli4" => Some(Self::MulInt4),
            "muli8" => Some(Self::MulInt8),
            "divi1" => Some(Self::DivInt1),
            "divi2" => Some(Self::DivInt2),
            "divi4" => Some(Self::DivInt4),
            "divi8" => Some(Self::DivInt8),
            "modi1" => Some(Self::ModInt1),
            "modi2" => Some(Self::ModInt2),
            "modi4" => Some(Self::ModInt4),
            "modi8" => Some(Self::ModInt8),
            "addf4" => Some(Self::AddFloat4),
            "addf8" => Some(Self::AddFloat8),
            "subf4" => Some(Self::SubFloat4),
            "subf8" => Some(Self::SubFloat8),
            "mulf4" => Some(Self::MulFloat4),
            "mulf8" => Some(Self::MulFloat8),
            "divf4" => Some(Self::DivFloat4),
            "divf8" => Some(Self::DivFloat8),
            "modf4" => Some(Self::ModFloat4),
            "modf8" => Some(Self::ModFloat8),
            "loadstatic1" => Some(Self::LoadStatic1),
            "loadstatic2" => Some(Self::LoadStatic2),
            "loadstatic4" => Some(Self::LoadStatic4),
            "loadstatic8" => Some(Self::LoadStatic8),
            "loadstaticn" => Some(Self::LoadStaticBytes),
            "loadconst1" => Some(Self::LoadConst1),
            "loadconst2" => Some(Self::LoadConst2),
            "loadconst4" => Some(Self::LoadConst4),
            "loadconst8" => Some(Self::LoadConst8),
            "loadconstn" => Some(Self::LoadConstBytes),
            "load1" => Some(Self::Load1),
            "load2" => Some(Self::Load2),
            "load4" => Some(Self::Load4),
            "load8" => Some(Self::Load8),
            "loadn" => Some(Self::LoadBytes),
            "virtualconsttoreal" => Some(Self::VirtualConstToReal),
            "virtualtoreal" => Some(Self::VirtualToReal),
            "store1" => Some(Self::Store1),
            "store2" => Some(Self::Store2),
            "store4" => Some(Self::Store4),
            "store8" => Some(Self::Store8),
            "storen" => Some(Self::StoreBytes),
            "memmove1" => Some(Self::Memmove1),
            "memmove2" => Some(Self::Memmove2),
            "memmove4" => Some(Self::Memmove4),
            "memmove8" => Some(Self::Memmove8),
            "memmoven" => Some(Self::MemmoveBytes),
            "malloc" => Some(Self::Malloc),
            "realloc" => Some(Self::Realloc),
            "free" => Some(Self::Free),
            "intr" => Some(Self::Intr),
            "exit" => Some(Self::Exit),
            "jumpconst" => Some(Self::JumpConst),
            "jump" => Some(Self::Jump),
            "jumpnzconst1" => Some(Self::JumpNotZeroConst1),
            "jumpnzconst2" => Some(Self::JumpNotZeroConst2),
            "jumpnzconst4" => Some(Self::JumpNotZeroConst4),
            "jumpnzconst8" => Some(Self::JumpNotZeroConst8),
            "jumpnz1" => Some(Self::JumpNotZero1),
            "jumpnz2" => Some(Self::JumpNotZero2),
            "jumpnz4" => Some(Self::JumpNotZero4),
            "jumpnz8" => Some(Self::JumpNotZero8),
            "jumpzconst1" => Some(Self::JumpZeroConst1),
            "jumpzconst2" => Some(Self::JumpZeroConst2),
            "jumpzconst4" => Some(Self::JumpZeroConst4),
            "jumpzconst8" => Some(Self::JumpZeroConst8),
            "jumpz1" => Some(Self::JumpZero1),
            "jumpz2" => Some(Self::JumpZero2),
            "jumpz4" => Some(Self::JumpZero4),
            "jumpz8" => Some(Self::JumpZero8),
            "nop" => Some(Self::Nop),
            
            _ => None
        }
    }

}


#[repr(u8)]
pub enum Interrupts {
    Write,
}

impl From<u8> for Interrupts {
    /// Convert a byte to an interrupt code. An invalid interrupt code will result in undefined behavior.
    fn from(byte: u8) -> Self {
        unsafe {
            mem::transmute(byte)
        }
    }
}

