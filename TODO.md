# HiVM

Project Description

<em>[TODO.md spec & Kanban Board](https://bit.ly/3fCwKfM)</em>

### Todo

- [ ] implement io interface through interrupts  
- [ ] we may incur in alignment problems when interpreting random memory addresses as *T. Testing is needed  
- [ ] write an assembler. advanced parsing would be nice to have. a tokenizer is thus required instead of a simple fixed argument table  
- [ ] maybe add a few registers to store frequently used variables  
- [ ] differentiate an optimized execution function and a safe execution function. the optimized execution function skips some safety checks like memory bounds  
- [ ] add verbose mode  

### In Progress

- [ ] add interrupts. some interrupts are predefined, other interrupts cause to jump to a specific memory address and execute from there  

### Done ✓

- [x] Differentiate between variable stack and operation stack  
- [x] update documentation with the new stack  
- [x] add proxy allocator  

