use static_assertions::const_assert_eq;

use hivmlib::{ByteCodes, Interrupts, ByteCode};

use std::mem::{self, MaybeUninit};
use std::alloc;
use std::ptr;

pub type Address = usize;

#[derive(Default)]
pub struct VirtualAddress(Address);

const_assert_eq!(mem::size_of::<VirtualAddress>(), mem::size_of::<usize>());


/// Interprets the first 8 bytes of the given byte slice as an address.
#[inline]
unsafe fn read_address(bytes: *const u8) -> Address {
    *(bytes as *const Address)
}


struct Stack {
    /// Raw pointer to the top of the stack. Modifying this pointer will directly modify the stack.
    tos: *mut u8,
    stack: Box<[u8]>,
}

impl Stack {

    pub fn new(size: usize) -> Self {
        let mut stack = unsafe {
            mem::transmute::<Box<[MaybeUninit<u8>]>, Box<[u8]>>(
                vec![MaybeUninit::uninit(); size].into_boxed_slice()
            )
        };
        Self {
            tos: unsafe {
                stack.as_mut_ptr().add(stack.len())
            },
            stack
        }
    }


    pub unsafe fn tos(&self) -> *const u8 {
        self.tos
    }


    pub unsafe fn tos_mut(&mut self) -> *mut u8 {
        self.tos
    }


    pub fn peek_1(&self) -> u8 {
        unsafe {
            self.tos.byte_sub(mem::size_of::<u8>()).read()
        }
    }


    pub fn peek_2(&self) -> u16 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u16>()) as *const u16).read()
        }
    }


    pub fn peek_4(&self) -> u32 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u32>()) as *const u32).read()
        }
    }


    pub fn peek_8(&self) -> u64 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u64>()) as *const u64).read()
        }
    }


    pub fn peek_bytes(&self, count: usize) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.tos.byte_sub(count), count)
        }
    }


    pub fn push_1(&mut self, byte: u8) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u8>());
            self.tos.write(byte);
        }
    }


    pub fn push_2(&mut self, value: u16) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u16>());
            (self.tos as *mut u16).write(value);
        }
    }


    pub fn push_4(&mut self, value: u32) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u32>());
            (self.tos as *mut u32).write(value);
        }
    }


    pub fn push_8(&mut self, value: u64) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u64>());
            (self.tos as *mut u64).write(value);
        }
    }


    pub fn push_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            self.tos = self.tos.byte_sub(bytes.len());
            self.tos.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len())
        }
    }


    pub fn push_from(&mut self, src: *const u8, count: usize) {
        unsafe {
            self.tos = self.tos.byte_sub(count);
            self.tos.copy_from_nonoverlapping(src, count);
        }
    }


    pub fn pop_1(&mut self) -> u8 {
        unsafe {
            let value = self.tos.read();
            self.tos = self.tos.byte_add(mem::size_of::<u8>());
            value
        }
    }


    pub fn pop_2(&mut self) -> u16 {
        unsafe {
            let value = (self.tos as *const u16).read();
            self.tos = self.tos.byte_add(mem::size_of::<u16>());
            value
        }
    }


    pub fn pop_4(&mut self) -> u32 {
        unsafe {
            let value = (self.tos as *const u32).read();
            self.tos = self.tos.byte_add(mem::size_of::<u32>());
            value
        }
    }


    pub fn pop_8(&mut self) -> u64 {
        unsafe {
            let value = (self.tos as *const u64).read();
            self.tos = self.tos.byte_add(mem::size_of::<u64>());
            value
        }
    }


    pub fn pop_bytes(&mut self, count: usize) -> &[u8] {
        unsafe {
            let bytes = std::slice::from_raw_parts::<u8>(self.tos, count);
            self.tos = self.tos.byte_add(count);
            bytes
        }
    }

}


struct Program<'a> {

    code: ByteCode<'a>,
    // Index of the next instruction/byte in the code.
    program_counter: usize,

}

impl<'a> Program<'a> {

    pub fn new(code: ByteCode<'a>) -> Self {

        if code.len() < mem::size_of::<VirtualAddress>() {
            panic!("Missing entry point");
        }

        Self {
            program_counter: unsafe { read_address(code.as_ptr()) },
            code,
        }
    }


    pub fn fetch_instruction(&mut self) -> ByteCodes {
        let instruction = ByteCodes::from(self.code[self.program_counter]);
        self.program_counter += 1;
        instruction
    }


    pub fn fetch_1(&mut self) -> u8 {
        let byte = self.code[self.program_counter];
        self.program_counter += mem::size_of::<u8>();
        byte
    }


    pub fn fetch_2(&mut self) -> u16 {
        let value = unsafe {
            *(self.code.as_ptr() as *const u16)
        };
        self.program_counter += mem::size_of::<u16>();
        value
    }


    pub fn fetch_4(&mut self) -> u32 {
        let value = unsafe {
            *(self.code.as_ptr() as *const u32)
        };
        self.program_counter += mem::size_of::<u32>();
        value
    }


    pub fn fetch_8(&mut self) -> u64 {
        let value = unsafe {
            *(self.code.as_ptr() as *const u64)
        };
        self.program_counter += mem::size_of::<u64>();
        value
    }


    pub fn fetch_bytes(&mut self, count: usize) -> &[u8] {
        let bytes = &self.code[self.program_counter..self.program_counter + count];
        self.program_counter += count;
        bytes
    }


    pub fn get_static1(&self, address: VirtualAddress) -> u8 {
        self.code[address.0]
    }


    pub fn get_static2(&self, address: VirtualAddress) -> u16 {
        unsafe {
            *(self.code[address.0..].as_ptr() as *const u16)
        }
    }


    pub fn get_static4(&self, address: VirtualAddress) -> u32 {
        unsafe {
            *(self.code[address.0..].as_ptr() as *const u32)
        }
    }


    pub fn get_static8(&self, address: VirtualAddress) -> u64 {
        unsafe {
            *(self.code[address.0..].as_ptr() as *const u64)
        }
    }


    pub fn get_static_bytes(&self, address: VirtualAddress, size: usize) -> &[u8] {
        &self.code[address.0..address.0 + size]
    }


    pub fn virtual_to_real(&self, vaddress: VirtualAddress) -> Address {
        vaddress.0 + self.code.as_ptr() as Address
    }

}


pub struct VM {

    /// Operation stack. Stores the operands and results of operations.
    opstack: Stack,
    /// Variable stack. Stores the variables in the stack frame.
    varstack: Stack,

}

// 1 KB should be enough for the operation stack since it stores temporary values (operands and results) 
// which should not be too large anyway. When processing big chunks of data, we usually use pointers to the data
// instead of copying the whole data itself.
const OPSTACK_SIZE: usize = 1024; // 1 KB
const DEFAULT_VARSTACK_SIZE: usize = 1024 * 1024; // 1 MB

impl VM {

    /// Instantiate a new VM with a given stack size.
    pub fn new(stack_size: Option<usize>) -> Self {
        Self {
            varstack: Stack::new(stack_size.unwrap_or(DEFAULT_VARSTACK_SIZE)),
            opstack: Stack::new(OPSTACK_SIZE),
        }
    }


    pub fn run(&mut self, code: ByteCode<'_>) -> i32 {

        let mut program = Program::new(code);

        loop {

            let instruction = program.fetch_instruction();

            // This match statement will be implemented through an efficient jump table by the compiler. 
            // There's no need to implement a jump table manually.
            match instruction {

                ByteCodes::AddInt1 => {
                    let a = self.opstack.pop_1() as i8;
                    let b = self.opstack.pop_1() as i8;
                    self.opstack.push_1(a.wrapping_add(b) as u8);
                },
                ByteCodes::AddInt2 => {
                    let a = self.opstack.pop_2() as i16;
                    let b = self.opstack.pop_2() as i16;
                    self.opstack.push_2(a.wrapping_add(b) as u16);
                },
                ByteCodes::AddInt4 => {
                    let a = self.opstack.pop_4() as i32;
                    let b = self.opstack.pop_4() as i32;
                    self.opstack.push_4(a.wrapping_add(b) as u32);
                },
                ByteCodes::AddInt8 => {
                    let a = self.opstack.pop_8() as i64;
                    let b = self.opstack.pop_8() as i64;
                    self.opstack.push_8(a.wrapping_add(b) as u64);
                },
                ByteCodes::SubInt1 => {
                    let a = self.opstack.pop_1() as i8;
                    let b = self.opstack.pop_1() as i8;
                    self.opstack.push_1(a.wrapping_sub(b) as u8);
                },
                ByteCodes::SubInt2 => {
                    let a = self.opstack.pop_2() as i16;
                    let b = self.opstack.pop_2() as i16;
                    self.opstack.push_2(a.wrapping_sub(b) as u16);
                },
                ByteCodes::SubInt4 => {
                    let a = self.opstack.pop_4() as i32;
                    let b = self.opstack.pop_4() as i32;
                    self.opstack.push_4(a.wrapping_sub(b) as u32);
                },
                ByteCodes::SubInt8 => {
                    let a = self.opstack.pop_8() as i64;
                    let b = self.opstack.pop_8() as i64;
                    self.opstack.push_8(a.wrapping_sub(b) as u64);
                },
                ByteCodes::MulInt1 => {
                    let a = self.opstack.pop_1() as i8;
                    let b = self.opstack.pop_1() as i8;
                    self.opstack.push_1(a.wrapping_mul(b) as u8);
                },
                ByteCodes::MulInt2 => {
                    let a = self.opstack.pop_2() as i16;
                    let b = self.opstack.pop_2() as i16;
                    self.opstack.push_2(a.wrapping_mul(b) as u16);
                },
                ByteCodes::MulInt4 => {
                    let a = self.opstack.pop_4() as i32;
                    let b = self.opstack.pop_4() as i32;
                    self.opstack.push_4(a.wrapping_mul(b) as u32);
                },
                ByteCodes::MulInt8 => {
                    let a = self.opstack.pop_8() as i64;
                    let b = self.opstack.pop_8() as i64;
                    self.opstack.push_8(a.wrapping_mul(b) as u64);
                },
                ByteCodes::DivInt1 => {
                    let a = self.opstack.pop_1() as i8;
                    let b = self.opstack.pop_1() as i8;
                    self.opstack.push_1(a.wrapping_div(b) as u8);
                },
                ByteCodes::DivInt2 => {
                    let a = self.opstack.pop_2() as i16;
                    let b = self.opstack.pop_2() as i16;
                    self.opstack.push_2(a.wrapping_div(b) as u16);
                },
                ByteCodes::DivInt4 => {
                    let a = self.opstack.pop_4() as i32;
                    let b = self.opstack.pop_4() as i32;
                    self.opstack.push_4(a.wrapping_div(b) as u32);
                },
                ByteCodes::DivInt8 => {
                    let a = self.opstack.pop_8() as i64;
                    let b = self.opstack.pop_8() as i64;
                    self.opstack.push_8(a.wrapping_div(b) as u64);
                },
                ByteCodes::ModInt1 => {
                    let a = self.opstack.pop_1() as i8;
                    let b = self.opstack.pop_1() as i8;
                    self.opstack.push_1(a.wrapping_rem(b) as u8);
                },
                ByteCodes::ModInt2 => {
                    let a = self.opstack.pop_2() as i16;
                    let b = self.opstack.pop_2() as i16;
                    self.opstack.push_2(a.wrapping_rem(b) as u16);
                },
                ByteCodes::ModInt4 => {
                    let a = self.opstack.pop_4() as i32;
                    let b = self.opstack.pop_4() as i32;
                    self.opstack.push_4(a.wrapping_rem(b) as u32);
                },
                ByteCodes::ModInt8 => {
                    let a = self.opstack.pop_8() as i64;
                    let b = self.opstack.pop_8() as i64;
                    self.opstack.push_8(a.wrapping_rem(b) as u64);
                },

                ByteCodes::AddFloat4 => {
                    let a = self.opstack.pop_4() as f32;
                    let b = self.opstack.pop_4() as f32;
                    self.opstack.push_4((a + b) as u32);
                },
                ByteCodes::AddFloat8 => {
                    let a = self.opstack.pop_8() as f64;
                    let b = self.opstack.pop_8() as f64;
                    self.opstack.push_8((a + b) as u64);
                },
                ByteCodes::SubFloat4 => {
                    let a = self.opstack.pop_4() as f32;
                    let b = self.opstack.pop_4() as f32;
                    self.opstack.push_4((a - b) as u32);
                },
                ByteCodes::SubFloat8 => {
                    let a = self.opstack.pop_8() as f64;
                    let b = self.opstack.pop_8() as f64;
                    self.opstack.push_8((a - b) as u64);
                },
                ByteCodes::MulFloat4 => {
                    let a = self.opstack.pop_4() as f32;
                    let b = self.opstack.pop_4() as f32;
                    self.opstack.push_4((a * b) as u32);
                },
                ByteCodes::MulFloat8 => {
                    let a = self.opstack.pop_8() as f64;
                    let b = self.opstack.pop_8() as f64;
                    self.opstack.push_8((a * b) as u64);
                },
                ByteCodes::DivFloat4 => {
                    let a = self.opstack.pop_4() as f32;
                    let b = self.opstack.pop_4() as f32;
                    self.opstack.push_4((a / b) as u32);
                },
                ByteCodes::DivFloat8 => {
                    let a = self.opstack.pop_8() as f64;
                    let b = self.opstack.pop_8() as f64;
                    self.opstack.push_8((a / b) as u64);
                },
                ByteCodes::ModFloat4 => {
                    let a = self.opstack.pop_4() as f32;
                    let b = self.opstack.pop_4() as f32;
                    self.opstack.push_4((a % b) as u32);
                },
                ByteCodes::ModFloat8 => {
                    let a = self.opstack.pop_8() as f64;
                    let b = self.opstack.pop_8() as f64;
                    self.opstack.push_8((a % b) as u64);
                },

                ByteCodes::Memmove1 => {
                    let dest = self.opstack.pop_8() as *mut u8;
                    let src = self.opstack.pop_8() as *const u8;
                    unsafe {
                        dest.write(*src);
                    }
                },
                ByteCodes::Memmove2 => {
                    let dest = self.opstack.pop_8() as *mut u16;
                    let src = self.opstack.pop_8() as *const u16;
                    unsafe {
                        dest.write(*src)
                    }
                },
                ByteCodes::Memmove4 => {
                    let dest = self.opstack.pop_8() as *mut u32;
                    let src = self.opstack.pop_8() as *const u32;
                    unsafe {
                        dest.write(*src);
                    }
                },
                ByteCodes::Memmove8 => {
                    let dest = self.opstack.pop_8() as *mut u64;
                    let src = self.opstack.pop_8() as *const u64;
                    unsafe {
                        dest.write(*src);
                    }
                },
                ByteCodes::MemmoveBytes => {
                    let dest = self.opstack.pop_8() as *mut u8;
                    let src = self.opstack.pop_8() as *const u8;
                    let count = self.opstack.pop_8() as usize;
                    unsafe {
                        // Assume the memory regions don't overlap.
                        dest.copy_from_nonoverlapping(src, count);
                    }
                },

                ByteCodes::VirtualConstToReal => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    self.opstack.push_8(program.virtual_to_real(vsrc) as u64);
                },
                ByteCodes::VirtualToReal => {
                    // TODO: here we could avoid popping and pushing the stack pointer by directly writing to the stack.
                    let vsrc = VirtualAddress(self.opstack.pop_8() as Address);
                    self.opstack.push_8(program.virtual_to_real(vsrc) as u64);
                }

                ByteCodes::LoadStatic1 => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    self.opstack.push_1(program.get_static1(vsrc));
                },
                ByteCodes::LoadStatic2 => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    self.opstack.push_2(program.get_static2(vsrc));
                },
                ByteCodes::LoadStatic4 => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    self.opstack.push_4(program.get_static4(vsrc));
                },
                ByteCodes::LoadStatic8 => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    self.opstack.push_8(program.get_static8(vsrc));
                },
                ByteCodes::LoadStaticBytes => {
                    let vsrc = VirtualAddress(program.fetch_8() as Address);
                    let count = program.fetch_8() as usize;
                    self.opstack.push_bytes(program.get_static_bytes(vsrc, count));
                },

                ByteCodes::Load1 => {
                    let src = self.opstack.pop_8() as *const u8;
                    self.opstack.push_1(unsafe { *src });
                },
                ByteCodes::Load2 => {
                    let src = self.opstack.pop_8() as *const u8;
                    self.opstack.push_2(unsafe { *(src as *const u16) });
                },
                ByteCodes::Load4 => {
                    let src = self.opstack.pop_8() as *const u8;
                    self.opstack.push_4(unsafe { *(src as *const u32) });
                },
                ByteCodes::Load8 => {
                    let src = self.opstack.pop_8() as *const u8;
                    self.opstack.push_8(unsafe { *(src as *const u64) });
                },
                ByteCodes::LoadBytes => {
                    let src = self.opstack.pop_8() as *const u8;
                    let count = self.opstack.pop_8() as usize;
                    self.opstack.push_from(src, count);
                },
                
                ByteCodes::LoadConst1 => {
                    self.opstack.push_1(program.fetch_1());
                },
                ByteCodes::LoadConst2 => {
                    self.opstack.push_2(program.fetch_2());
                },
                ByteCodes::LoadConst4 => {
                    self.opstack.push_4(program.fetch_4());
                },
                ByteCodes::LoadConst8 => {
                    self.opstack.push_8(program.fetch_8());
                },
                ByteCodes::LoadConstBytes => {
                    let count = program.fetch_8() as usize;
                    self.opstack.push_bytes(program.fetch_bytes(count));
                },

                ByteCodes::Store1 => {
                    let dest = self.opstack.pop_8() as *mut u8;
                    unsafe {
                        dest.write(self.opstack.pop_1());
                    }
                },
                ByteCodes::Store2 => {
                    let dest = self.opstack.pop_8() as *mut u16;
                    unsafe {
                        dest.write(self.opstack.pop_2());
                    }
                },
                ByteCodes::Store4 => {
                    let dest = self.opstack.pop_8() as *mut u32;
                    unsafe {
                        dest.write(self.opstack.pop_4());
                    }
                },
                ByteCodes::Store8 => {
                    let dest = self.opstack.pop_8() as *mut u64;
                    unsafe {
                        dest.write(self.opstack.pop_8());
                    }
                },
                ByteCodes::StoreBytes => {
                    let dest = self.opstack.pop_8() as *mut u8;
                    let count = self.opstack.pop_8() as usize;
                    unsafe {
                        // Note that the stack and whatever memory is being written to must not overlap. It's the programmer's responsibility to ensure this.
                        dest.copy_from_nonoverlapping(
                            self.opstack.pop_bytes(count).as_ptr(), 
                            count
                        );
                    }
                },

                ByteCodes::Malloc => {
                    let size = self.opstack.pop_8() as usize;
                    let addr = unsafe {
                        match alloc::Layout::array::<u8>(size) {
                            Ok(layout) => {
                                alloc::alloc(layout)
                            },
                            Err(_) => {
                                ptr::null_mut()
                            }
                        }
                    };
                    self.opstack.push_8(addr as u64);
                },
                ByteCodes::Realloc => {
                    let addr = self.opstack.pop_8() as *mut u8;
                    let new_size = self.opstack.pop_8() as usize;
                    let new_addr = unsafe {
                        match alloc::Layout::array::<u8>(new_size) {
                            Ok(layout) => {
                                alloc::realloc(addr, layout, new_size)
                            },
                            Err(_) => {
                                ptr::null_mut()
                            }
                        }
                    };
                    self.opstack.push_8(new_addr as u64);
                },
                ByteCodes::Free => {
                    let addr = self.opstack.pop_8() as *mut u8;
                    unsafe {
                        alloc::dealloc(addr, alloc::Layout::new::<u8>());
                    }
                },

                ByteCodes::Exit => {
                    let code = self.opstack.pop_4() as i32;
                    return code;
                },

                ByteCodes::Intr => {
                    let intr_code = Interrupts::from(self.opstack.pop_1());
                    self.handle_interrupt(intr_code);
                },

                ByteCodes::Duplicate1 => {
                    self.opstack.push_1(
                        self.opstack.peek_1()
                    );
                },

                ByteCodes::Duplicate2 => {
                    self.opstack.push_2(
                        self.opstack.peek_2()
                    );
                },

                ByteCodes::Duplicate4 => {
                    self.opstack.push_4(
                        self.opstack.peek_4()
                    );
                },

                ByteCodes::Duplicate8 => {
                    self.opstack.push_8(
                        self.opstack.peek_8()
                    );
                },

                ByteCodes::DuplicateBytes => {
                    let count = self.opstack.pop_8() as usize;
                    let bytes = unsafe { std::slice::from_raw_parts(self.opstack.tos(), count) };
                    self.opstack.push_bytes(bytes);
                }

                ByteCodes::Nop => { /* Do nothing */ },

            }
        }
    }


    fn handle_interrupt(&mut self, intr_code: Interrupts) {
        match intr_code {
            Interrupts::Write => todo!(),
        }
    }

}

