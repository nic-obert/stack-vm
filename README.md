# HiVM - High Level Virtual Machine

- [HiVM - High Level Virtual Machine](#hivm---high-level-virtual-machine)
  - [VM Design](#vm-design)
    - [Stack](#stack)
    - [Heap](#heap)
    - [Program counter](#program-counter)
    - [Stack pointer](#stack-pointer)
    - [Program space](#program-space)
  - [License](#license)

This is a high level stack-based virtual machine that is designed to be simple and easy to understand. It works on a reduced instruction set, compared to regular processors (like x86 which has roughly 80 instructions).

HiVM is a project intended to be a learning journey into virtual machines and compiler design. It is not intended to be used in production environments.

## VM Design

HiVM is meant to be a relatively high-level VM, meaning the programmer does not direclty set the registers. In fact, there are no general purpose registers in HiVM since emulated registers, which are implemented as an array of 64-bit numbers, are generally not faster than an array lookup to access the stack.  
However, on modern CPUs we may gain some advantage by using registers to store frequently used data.

HiVM's memory is an interface between the program and the host machine/OS. Pointers inside HiVM will point to the data in the host memory. This means that HiVM pointers are not virtualized. The direct HiVM pointer to host memory correspondence is used for fast memory access, which would otherwise be implemented through an index operation.

Now, since HiVM programs have direct access to the host memory, there is the possibility of accessing the host's resources. It is up to the host OS to prevent illegal memory accesses and segfault accordingly.

### Stack

The core component of HiVM is the stack. The stack is a contiguous section of memory used to store local variables, return values, function call arguments, and such.
The stack has a fixed size that is determined at the start of the program and cannot change at runtime.

Popping form and pushing onto the stack are fast operations since there's no allocation involved. The stack pointer is incremented or decremented to keep track of the top of the stack.

The stack grows top-to-bottom. Popping form the stack increments the stack pointer while pushing onto the stack decrements the stack pointer.

### Heap

HiVM implements a heap memory by acting as an interface between the program and the host memory. Becuse of this, the heap may not be contiguous and doesn't have a fixed size. Memory blocks can be allocated and deallocated through interrupts, which is one of the high-level features of HiVM.

### Program counter

A specific register that stores the next instruction in the program. The program counter is altered whenever a jump is performed.

Registers should not be accessed by the program.

### Stack pointer

A specific register that stores the address of the top of the stack (topmost byte). This register may be altered to efficiently push onto or pop from the stack.

Because HiVM doesn't virtualize pointers, the stack pointer points to the TOS relative to the host memory.

Registers should not be accessed by the program.

### Program space

A contiguous section of memory used to store the program's instructions. This memory is read-only and is set once when the program is first loaded into memory.

The first 8 bytes of the program space are interpreted as the address of the program entry point.

Note that pointers in program space are unaware of the underlying host memory. Because of this, when accessing an address in program space (e.g. static data), the program needs to offset the pointer by the address of the program space.  
Pointers to static data are treated differently than pointers to regular data (stack and heap). To access static data a dedicated instruction may be used.

Offsetting static data upon accessing may seem a preventable overhead, but a virtualized memory space would need to be indexed, resulting in an inevitable offsetting operation. By only de-virtualizing pointers to static data at runtime and using unvirtualized pointers for regular data, HiVM reduces the otherwise inevitable overhead of accessing virtualized memory in the majority of cases.

It's up to the programmer (or compiler) to handle the virtualized static data pointers correctly by using the appropriate instructions and by not mixing virtual pointers with raw host pointers.

## License

This project and all related files are published under the [MIT License](LICENSE).
