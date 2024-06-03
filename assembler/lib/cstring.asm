

include "io.asm"


@cstrlen

; ...
; arg 8 bytes
; ret 8 bytes
; -> 

; calculate the stack address of the argument
    loadsp
    loadc8 8
    addi8

; load the argument (char*)
    load8
    dup8

; current char pointer
    !r2
    store8

; set immutable original char* to compare later
    !r3
    store8


    @loop
    ; load the current char
        !ldr2
        load1

    ; compare with \0
        loadc1 '\0'
        subi1 

    ; update the current char register
        !ldr2
        loadc8 1
        addi8
        !r2
        store8

        jnzc1 loop

    ; calculate the string length
    !ldr2
    !ldr3
    subi8

    !r1
    store8

    ret
    
