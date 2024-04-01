use static_assertions::const_assert_eq;

use crate::bytecode::ByteCodes;

use std::mem::{self, MaybeUninit};

pub type Address = usize;

#[derive(Default)]
pub struct VirtualAddress(Address);

const_assert_eq!(mem::size_of::<VirtualAddress>(), mem::size_of::<usize>());

pub type ByteCode<'a> = &'a [u8];


/// Interprets the first 8 bytes of the given byte slice as an address.
#[inline]
unsafe fn read_address(bytes: *const u8) -> Address {
    *(bytes as *const Address)
}


struct Stack {
    /// Raw pointer to the top of the stack. Modifying this pointer will directly modify the stack.
    tos: *mut u8,
    stack: Box<[u8]>
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
            let bytes = std::slice::from_raw_parts(self.tos, count);
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


}


pub struct VM {

    stack: Stack,

}

impl VM {

    /// Instantiate a new VM with a given stack size.
    pub fn new(stack_size: usize) -> Self {
        Self {
            stack: Stack::new(stack_size),
        }
    }



    pub fn run(&mut self, code: ByteCode<'_>) {

        let mut program = Program::new(code);

        loop {

            let instruction = program.fetch_instruction();

            // This match statement will be implemented through an efficient jump table by the compiler. 
            // There's no need to implement a jump table manually.
            match instruction {

                ByteCodes::LoadStatic => todo!(),
                ByteCodes::VirtualToReal => todo!(),

                ByteCodes::Load1 => {
                    let src = self.stack.pop_8() as *const u8;
                    self.stack.push_1(unsafe { *src });
                },
                ByteCodes::Load2 => {
                    let src = self.stack.pop_8() as *const u8;
                    self.stack.push_2(unsafe { *(src as *const u16) });
                },
                ByteCodes::Load4 => {
                    let src = self.stack.pop_8() as *const u8;
                    self.stack.push_4(unsafe { *(src as *const u32) });
                },
                ByteCodes::Load8 => {
                    let src = self.stack.pop_8() as *const u8;
                    self.stack.push_8(unsafe { *(src as *const u64) });
                },
                ByteCodes::LoadBytes => {
                    let src = self.stack.pop_8() as *const u8;
                    let count = self.stack.pop_8() as usize;
                    self.stack.push_from(src, count);
                },
                
                ByteCodes::LoadConst1 => {
                    self.stack.push_1(program.fetch_1());
                },
                ByteCodes::LoadConst2 => {
                    self.stack.push_2(program.fetch_2());
                },
                ByteCodes::LoadConst4 => {
                    self.stack.push_4(program.fetch_4());
                },
                ByteCodes::LoadConst8 => {
                    self.stack.push_8(program.fetch_8());
                },
                ByteCodes::LoadConstBytes => {
                    let count = program.fetch_8() as usize;
                    self.stack.push_bytes(program.fetch_bytes(count));
                },

                ByteCodes::Nop => { /* Do nothing */ },
                

            }
        }
    }

}
