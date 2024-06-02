

; This macro must be the first instruction to be called by the program
%init_reglib

; initialize the registers to 0

    loadc8 0
    dup8
    dup8
    dup8

    !r1
    store8

    !r2
    store8

    !r3
    store8

    !r3
    store8   

%endmacro


%r1

    loadsb

%endmacro




%r2

    loadsb
    loadc8 8
    addi8

%endmacro


%r3

    loadsb
    loadc8 16
    addi8

%endmacro


%r4

    loadsb
    loadc8 24
    addi8

%endmacro


%ldr1

    !r1
    load8

%endmacro


%ldr2

    !r2
    load8

%endmacro


%ldr3

    !r3
    load8

%endmacro


%ldr4

    !r4
    load8

%endmacro
