
use hivmlib::{Address, ByteCode, ByteCodes, ErrorCodes, Interrupts, VirtualAddress};

use std::io::Read;
use std::mem::{self, MaybeUninit};
use std::{alloc, io, slice};
use std::ptr;


struct Stack {
    /// Raw pointer to the top of the stack. Modifying this pointer will directly modify the stack.
    tos: *mut u8,
    /// Owned pointer to the stack. The stack is mutated thorugh the `tos` pointer.
    _stack: Box<[u8]>,
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
            _stack: stack
        }
    }


    pub unsafe fn tos(&self) -> *const u8 {
        self.tos
    }


    // pub unsafe fn tos_mut(&mut self) -> *mut u8 {
    //     self.tos
    // }


    pub fn peek_1(&self) -> u8 {
        unsafe {
            self.tos.byte_sub(mem::size_of::<u8>()).read_unaligned()
        }
    }


    pub fn peek_2(&self) -> u16 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u16>()) as *const u16).read_unaligned()
        }
    }


    pub fn peek_4(&self) -> u32 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u32>()) as *const u32).read_unaligned()
        }
    }


    pub fn peek_8(&self) -> u64 {
        unsafe {
            (self.tos.byte_sub(mem::size_of::<u64>()) as *const u64).read_unaligned()
        }
    }


    // pub fn peek_bytes(&self, count: usize) -> &[u8] {
    //     unsafe {
    //         std::slice::from_raw_parts(self.tos.byte_sub(count), count)
    //     }
    // }


    pub fn push_1(&mut self, byte: u8) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u8>());
            self.tos.write_unaligned(byte);
        }
    }


    pub fn push_2(&mut self, value: u16) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u16>());
            (self.tos as *mut u16).write_unaligned(value);
        }
    }


    pub fn push_4(&mut self, value: u32) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u32>());
            (self.tos as *mut u32).write_unaligned(value);
        }
    }


    pub fn push_8(&mut self, value: u64) {
        unsafe {
            self.tos = self.tos.byte_sub(mem::size_of::<u64>());
            (self.tos as *mut u64).write_unaligned(value);
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
            let value = self.tos.read_unaligned();
            self.tos = self.tos.byte_add(mem::size_of::<u8>());
            value
        }
    }


    pub fn pop_2(&mut self) -> u16 {
        unsafe {
            let value = (self.tos as *const u16).read_unaligned();
            self.tos = self.tos.byte_add(mem::size_of::<u16>());
            value
        }
    }


    pub fn pop_4(&mut self) -> u32 {
        unsafe {
            let value = (self.tos as *const u32).read_unaligned();
            self.tos = self.tos.byte_add(mem::size_of::<u32>());
            value
        }
    }


    pub fn pop_8(&mut self) -> u64 {
        unsafe {
            let value = (self.tos as *const u64).read_unaligned();
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
    program_counter: VirtualAddress,

}

impl<'a> Program<'a> {

    pub fn new(code: ByteCode<'a>) -> Self {

        if code.len() < mem::size_of::<VirtualAddress>() {
            panic!("Missing entry point");
        }

        Self {
            program_counter: VirtualAddress(0),
            code,
        }
    }


    pub fn jump_to(&mut self, target: VirtualAddress) {
        self.program_counter = target;
    }


    pub fn fetch_instruction(&mut self) -> Option<ByteCodes> {
        let instruction = ByteCodes::from(*self.code.get(self.program_counter.0)?);
        self.program_counter.0 += 1;
        Some(instruction)
    }


    pub fn fetch_1(&mut self) -> u8 {
        let byte = self.code[self.program_counter.0];
        self.program_counter.0 += 1;
        byte
    }


    pub fn fetch_2(&mut self) -> u16 {
        let value = unsafe {
            ((self.code.as_ptr().add(self.program_counter.0)) as *const u16).read_unaligned()
        };
        self.program_counter.0 += 2;
        value
    }


    pub fn fetch_4(&mut self) -> u32 {
        let value = unsafe {
            (self.code.as_ptr().add(self.program_counter.0) as *const u32).read_unaligned()
        };
        self.program_counter.0 += 4;
        value
    }


    pub fn fetch_8(&mut self) -> u64 {
        let value = unsafe {
            (self.code.as_ptr().add(self.program_counter.0) as *const u64).read_unaligned()
        };
        self.program_counter.0 += 8;
        value
    }


    pub fn fetch_bytes(&mut self, count: usize) -> &[u8] {
        let bytes = &self.code[self.program_counter.0..self.program_counter.0 + count];
        self.program_counter.0 += count;
        bytes
    }


    pub fn get_static1(&self, address: VirtualAddress) -> u8 {
        self.code[address.0]
    }


    pub fn get_static2(&self, address: VirtualAddress) -> u16 {
        unsafe {
            (self.code[address.0..].as_ptr() as *const u16).read_unaligned()
        }
    }


    pub fn get_static4(&self, address: VirtualAddress) -> u32 {
        unsafe {
            (self.code[address.0..].as_ptr() as *const u32).read_unaligned()
        }
    }


    pub fn get_static8(&self, address: VirtualAddress) -> u64 {
        unsafe {
            (self.code[address.0..].as_ptr() as *const u64).read_unaligned()
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
    /// Stores the last error code.
    error_code: ErrorCodes

}

// 1 KB should be enough for the operation stack since it stores temporary values (operands and results) 
// which should not be too large anyway. When processing big chunks of data, we usually use pointers to the data
// instead of copying the whole data itself.
const DEFAULT_OPSTACK_SIZE: usize = 1024; // 1 KB

impl VM {

    /// Instantiate a new VM with a given stack size.
    pub fn new(opstack_size: Option<usize>) -> Self {
        Self {
            opstack: Stack::new(opstack_size.unwrap_or(DEFAULT_OPSTACK_SIZE)),
            error_code: ErrorCodes::NoError
        }
    }


    pub fn run(&mut self, code: ByteCode<'_>) -> ErrorCodes {

        let mut program = Program::new(code);

        while let Some(instruction) = program.fetch_instruction() {

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
                    let exit_code = self.opstack.pop_4() as i32;
                    return ErrorCodes::from(exit_code);
                },

                ByteCodes::Intr => {
                    let intr_code = Interrupts::from(self.opstack.pop_1());
                    self.handle_interrupt(intr_code, &mut program);
                },

                ByteCodes::IntrConst => {
                    let intr_code = Interrupts::from(program.fetch_1());
                    self.handle_interrupt(intr_code, &mut program);
                },

                ByteCodes::ReadError => {
                    self.opstack.push_4(self.error_code as u32);
                },

                ByteCodes::SetErrorConst => {
                    let error_code = program.fetch_4() as i32;
                    self.error_code = ErrorCodes::from(error_code);
                },

                ByteCodes::SetError => {
                    let error_code = self.opstack.pop_4() as i32;
                    self.error_code = ErrorCodes::from(error_code);
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
                },

                ByteCodes::JumpConst => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    program.jump_to(target);
                },

                ByteCodes::Jump => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    program.jump_to(target);
                },

                ByteCodes::JumpNotZeroConst1 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_1();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZeroConst2 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_2();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZeroConst4 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_4();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZeroConst8 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_8();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZero1 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_1();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZero2 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_2();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZero4 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_4();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNotZero8 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_8();
                    if condition != 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZeroConst1 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_1();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZeroConst2 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_2();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZeroConst4 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_4();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZeroConst8 => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    let condition = self.opstack.pop_8();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZero1 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_1();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZero2 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_2();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZero4 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_4();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpZero8 => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    let condition = self.opstack.pop_8();
                    if condition == 0 {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpError => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    if !matches!(self.error_code, ErrorCodes::NoError) {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNoError => {
                    let target = VirtualAddress(self.opstack.pop_8() as usize);
                    if matches!(self.error_code, ErrorCodes::NoError) {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpErrorConst => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    if !matches!(self.error_code, ErrorCodes::NoError) {
                        program.jump_to(target);
                    }
                },

                ByteCodes::JumpNoErrorConst => {
                    let target = VirtualAddress(program.fetch_8() as usize);
                    if matches!(self.error_code, ErrorCodes::NoError) {
                        program.jump_to(target);
                    }
                },

                ByteCodes::Nop => { /* Do nothing */ },

            }
        }

        // The program has no more instruction to execute and an exit code was not provided.
        // Assume the program ended successfully.
        self.error_code
    }


    fn handle_interrupt(&mut self, intr_code: Interrupts, program: &mut Program) {

        match intr_code {

            Interrupts::Print1 => {
                let value = self.opstack.pop_1();
                print!("{}", value);
            },
            Interrupts::Print2 => {
                let value = self.opstack.pop_2();
                print!("{}", value);
            },
            Interrupts::Print4 => {
                let value = self.opstack.pop_4();
                print!("{}", value);
            },
            Interrupts::Print8 => {
                let value = self.opstack.pop_8();
                print!("{}", value);
            },
            Interrupts::PrintBytes => {
                let count = self.opstack.pop_8() as usize;
                let bytes_addr = self.opstack.pop_8() as *const u8;
                let bytes = unsafe {
                    slice::from_raw_parts(bytes_addr, count)
                };
                print!("{:?}", bytes);
            },
            Interrupts::PrintChar => {
                let value = self.opstack.pop_1();
                print!("{}", value as char);
            },
            Interrupts::PrintString => {
                let length = self.opstack.pop_8() as usize;
                let str_addr = self.opstack.pop_8() as *const u8;
                // Use unchecked because it's the programmer's responsibility to ensure the string is valid
                unsafe {
                    let string = slice::from_raw_parts(str_addr, length);
                    let string = std::str::from_utf8_unchecked(string);
                    print!("{}", string);
                }
            },
            Interrupts::PrintStaticBytes => {
                let count = self.opstack.pop_8() as usize;
                let bytes_vaddr = VirtualAddress(self.opstack.pop_8() as usize);
                let bytes = program.get_static_bytes(bytes_vaddr, count);
                print!("{:?}", bytes);
            },
            Interrupts::PrintStaticString => {
                let length = self.opstack.pop_8() as usize;
                let str_vaddr = VirtualAddress(self.opstack.pop_8() as usize);
                let string = unsafe {
                    std::str::from_utf8_unchecked(
                        program.get_static_bytes(str_vaddr, length)
                    )
                };
                print!("{}", string);
            },
            Interrupts::ReadBytes => {
                let n = self.opstack.pop_8() as usize;
                let mut buf = Vec::with_capacity(n);
                if let Err(err) = io::stdin().read_exact(&mut buf) {
                   self.error_code = match err.kind() {
                        io::ErrorKind::UnexpectedEof => ErrorCodes::UnexpectedEOF,
                        _ => ErrorCodes::GenericError,
                    }
                } else {
                    self.opstack.push_bytes(&buf);
                }
            },
            Interrupts::ReadAll => {
                let mut buf = Vec::new();
                match io::stdin().read_to_end(&mut buf) {
                    Ok(bytes_read) => {
                        self.opstack.push_bytes(&buf);
                        self.opstack.push_8(bytes_read as u64);
                    },
                    Err(_err) => self.error_code = ErrorCodes::GenericError
                }
            },

        }
    }

}

