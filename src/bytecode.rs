

/// HiVM instructions. Each instruction is represented by one byte.
#[repr(u8)]
pub enum ByteCodes {

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

    Malloc,
    Realloc,
    Free,

    Exit,

    /// No operation. Do nothing for this cycle.
    Nop

}

impl From<u8> for ByteCodes {
    fn from(byte: u8) -> Self {
        unsafe {
            std::mem::transmute(byte)
        }
    }

}

