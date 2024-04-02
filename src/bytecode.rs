
/// Byte codes for the HiVM. Each instruction is a single byte.
#[repr(u8)]
pub enum ByteCodes {

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

    Malloc,
    Realloc,
    Free,

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

