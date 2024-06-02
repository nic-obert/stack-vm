
include "reglib.asm"
include "archlib.asm"


.text

    !init_reglib

    loadc8 104
    !r2
    store8

    !r2
    load1
    intrconst !PRINT_CHAR_INTR

    loadc1 10
    intrconst !PRINT_CHAR_INTR

    exit
    

