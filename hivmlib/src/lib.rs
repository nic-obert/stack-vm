use std::mem;

use static_assertions::{const_assert, const_assert_eq};


pub type Address = usize;
pub const ADDRESS_SIZE: usize = mem::size_of::<Address>();
pub const INSTRUCTION_SIZE: usize = 1;

#[derive(Default)]
pub struct VirtualAddress(pub Address);

impl VirtualAddress {

    pub fn to_le_bytes(self) -> [u8; mem::size_of::<Address>()] {
        self.0.to_le_bytes()
    }

}

const_assert_eq!(mem::size_of::<VirtualAddress>(), mem::size_of::<usize>());


pub type ByteCode<'a> = &'a [u8];


macro_rules! declare_instructions {
    ($($name:ident $asm_name:ident),+) => {
        
/// HiVM instructions. Each instruction is represented by one byte.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ByteCodes {
    $($name),+
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
            $(stringify!($asm_name) => Some(Self::$name),)+
            _ => None
        }
    }

}

    };
}

declare_instructions! {

    AddInt1 addi1,
    AddInt2 addi2,
    AddInt4 addi4,
    AddInt8 addi8,
    SubInt1 subi1,
    SubInt2 subi2,
    SubInt4 subi4,
    SubInt8 subi8,
    MulInt1 muli1,
    MulInt2 muli2,
    MulInt4 muli4,
    MulInt8 muli8,
    DivInt1 divi1,
    DivInt2 divi2,
    DivInt4 divi4,
    DivInt8 divi8,
    ModInt1 modi1,
    ModInt2 modi2,
    ModInt4 modi4,
    ModInt8 modi8,

    AddFloat4 addf4,
    AddFloat8 addf8,
    SubFloat4 subf4,
    SubFloat8 subf8,
    MulFloat4 mulf4,
    MulFloat8 mulf8,
    DivFloat4 divf4,
    DivFloat8 divf8,
    ModFloat4 modf4,
    ModFloat8 modf8,

    LoadStatic1 loadstatic1,
    LoadStatic2 loadstatic2,
    LoadStatic4 loadstatic4,
    LoadStatic8 loadstatic8,
    LoadStaticBytes loadstaticn,

    LoadConst1 loadconst1,
    LoadConst2 loadconst2,
    LoadConst4 loadconst4,
    LoadConst8 loadconst8,
    LoadConstBytes loadconstn,

    Load1 load1,
    Load2 load2,
    Load4 load4,
    Load8 load8,
    LoadBytes loadn,
    
    VirtualConstToReal virtualconsttoreal,
    VirtualToReal virtualtoreal,

    Store1 store1,
    Store2 store2,
    Store4 store4,
    Store8 store8,
    StoreBytes storen,

    Memmove1 memmove1,
    Memmove2 memmove2,
    Memmove4 memmove4,
    Memmove8 memmove8,
    MemmoveBytes memmoven,

    Duplicate1 dup1,
    Duplicate2 dup2,
    Duplicate4 dup4,
    Duplicate8 dup8,
    DuplicateBytes dupn,

    Malloc malloc,
    Realloc realloc,
    Free free,

    Intr intr,
    IntrConst intrconst,

    Exit exit,

    JumpConst jmpconst,
    Jump jmp,

    JumpNotZeroConst1 jnzconst1,
    JumpNotZeroConst2 jnzconst2,
    JumpNotZeroConst4 jnzconst4,
    JumpNotZeroConst8 jnzconst8,
    JumpNotZero1 jnz1,
    JumpNotZero2 jnz2,
    JumpNotZero4 jnz4,
    JumpNotZero8 jnz8,
    JumpZeroConst1 jzconst1,
    JumpZeroConst2 jzconst2,
    JumpZeroConst4 jzconst4,
    JumpZeroConst8 jzconst8,
    JumpZero1 jz1,
    JumpZero2 jz2,
    JumpZero4 jz4,
    JumpZero8 jz8,

    Nop nop

}

const_assert!(mem::size_of::<ByteCodes>() == INSTRUCTION_SIZE);


#[repr(u8)]
pub enum Interrupts {
    Print1 = 0,
    Print2,
    Print4,
    Print8,
    PrintBytes,
    PrintChar,
    PrintString,
}

impl From<u8> for Interrupts {
    /// Convert a byte to an interrupt code. An invalid interrupt code will result in undefined behavior.
    fn from(byte: u8) -> Self {
        unsafe {
            mem::transmute(byte)
        }
    }
}

