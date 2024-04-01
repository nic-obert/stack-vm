
/// Byte codes for the HiVM. Each instruction is a single byte.
#[repr(u8)]
pub enum ByteCodes {

    /// Load a static value from the program's static data section given its TOS virtual address.
    LoadStatic,

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
    
    VirtualToReal,

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

