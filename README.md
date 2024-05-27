# Stack VM

- [Stack VM](#stack-vm)
  - [VM Design](#vm-design)
    - [Operation Stack](#operation-stack)
    - [Heap](#heap)
    - [Program counter](#program-counter)
    - [Stack pointer](#stack-pointer)
    - [Program space](#program-space)
  - [License](#license)

This is a relatively high-level 64-bit stack-based virtual machine that is designed to be simple and easy to understand. It works on a size-comprehensive stack-based instruction set and allows granular control over sized operations.

Stack VM is a project intended to be a learning journey into virtual machines, assemblers, and compiler design. A stack-only vm is a challenge both to implement and to write programs for. Stack machines are also usually slower than register-based machines because of the higher number of instructions needed to perform even the simplest operations, and thus result in an increased number of CPU cycles. While register-based architectures can store temporary values into registers, a stack-based architecture has the stack as the only storage option. Because of this, stack machines must repetitively push onto and pop from the stack, calculate TOS (top of stack) offsets, and duplicate values just to access a stored variable.

This project is not intended to be used in production environments.

## VM Design

There are no general purpose registers to be used by the programmer and the only storage options are the built-in stack and heap.

The VM's memory is an interface between the program and the host machine/OS. Pointers inside the VM will point to the data in the host memory. This means that Stack VM pointers are not virtualized. The direct pointer to host memory correspondence is used for fast memory access, which would otherwise be implemented through an array index operation and result in more CPU cycles.

Now, since Stack VM programs have direct access to the host memory, there is the possibility of accessing the host's resources. It is up to the host OS to prevent illegal memory accesses and segfault accordingly.

### Operation Stack

The operation stack is a contiguous section of memory used to store temporary vales related to ongoing operations (operands and results).  
The stack has a fixed size and grows from top-to-bottom.

### Heap

Stack VM implements a heap memory by acting as an interface between the program and the host memory. Because of this, the heap may not be contiguous and doesn't have a fixed size. Memory blocks can be allocated and deallocated through interrupts, which is one of the high-level features of Stack VM.

### Program counter

A specific internal register that stores the next instruction in the program. The program counter is altered whenever a jump is performed.

The program counter cannot be directly mutated by running programs, but only through jump instructions.

### Stack pointer

A specific internal register that stores the address of the top of the stack (topmost byte). This register may be altered to efficiently push onto or pop from the stack.

Because Stack VM doesn't virtualize pointers, the stack pointer points to the TOS relative to the host memory.

The stack pointer cannot be directly mutated by the program.

### Program space

A contiguous section of memory used to store the program's instructions. This memory is read-only and is set once when the program is first loaded into memory.

The VM starts executing the program code from the first byte onwards. So, the first instruction that will be executed by the VM is the first byte of the program space.

Note that pointers in program space are unaware of the underlying host memory. Because of this, when accessing an address in program space (e.g. static data or labels), the program needs to offset the pointer by the address of the program space. This is done through the `vtr` (virtual to real) built-in instruction.  
Pointers to static data are treated differently than pointers to regular data (stack and heap). To access static data a dedicated instruction may be used.

Offsetting static data upon accessing may seem a preventable overhead, but a virtualized memory space would need to be indexed, resulting in an inevitable offsetting operation. By only de-virtualizing pointers to static data at runtime and using real pointers for regular data, Stack VM reduces the otherwise inevitable overhead of accessing virtualized memory in the majority of cases, assuming the program mostly accesses non-static data.

It's up to the programmer (or compiler) to handle the virtualized static data pointers correctly by using the appropriate instructions and by not mixing virtual pointers with host pointers.

## License

This project and all related files are published under the [MIT License](LICENSE).
