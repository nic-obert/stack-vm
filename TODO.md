# HiVM

Project Description

<em>[TODO.md spec & Kanban Board](https://bit.ly/3fCwKfM)</em>

### Todo

- [ ] refactor the project and divide tasks into different files  
- [ ] write assembly documentation  
- [ ] implement assembly sections  
- [ ] implement macros  
- [ ] implement a primitive assembly module system like #include in C  
- [ ] implement in-place math in assembly for constants  
- [ ] implement io interface through interrupts  
- [ ] we may incur in alignment problems when interpreting random memory addresses as *T. Testing is needed  
- [ ] maybe add a few registers to store frequently used variables  
- [ ] differentiate an optimized execution function and a safe execution function. the optimized execution function skips some safety checks like memory bounds  
- [ ] add verbose mode  

### In Progress

- [ ] write an assembler. advanced parsing would be nice to have. a tokenizer is thus required instead of a simple fixed argument table  
- [ ] add interrupts. some interrupts are predefined, other interrupts cause to jump to a specific memory address and execute from there  

### Done ✓

- [x] Differentiate between variable stack and operation stack  
- [x] update documentation with the new stack  
- [x] add proxy allocator  

