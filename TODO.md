# HiVM

Project Description

<em>[TODO.md spec & Kanban Board](https://bit.ly/3fCwKfM)</em>

### Todo

- [ ] implement value definition macros  
- [ ] write a disassembler  
- [ ] write assembly documentation  
- [ ] implement a primitive assembly module system like #include in C  
- [ ] implement in-place math in assembly for constants  
- [ ] differentiate an optimized vm execution function and a safe execution function. the optimized execution function skips some safety checks like memory bounds  
- [ ] add verbose mode  

### In Progress

- [ ] implement io interface through interrupts  

### Done ✓

- [x] implement macros  
- [x] add interrupts. some interrupts are predefined, other interrupts cause to jump to a specific memory address and execute from there  
- [x] we may incur in alignment problems when interpreting random memory addresses as *T. Testing is needed  
- [x] implement assembly sections  
- [x] maybe the section enum should not exist. section names should be arbitrary maybe. the only exception would be the .text section if an entry point is required. maybe we should specify a .entry or .start section that tells the assembler that is the entry point  
- [x] write an assembler. advanced parsing would be nice to have. a tokenizer is thus required instead of a simple fixed argument table  
- [x] refactor the project and divide tasks into different files  
- [x] Differentiate between variable stack and operation stack  
- [x] update documentation with the new stack  
- [x] add proxy allocator  

