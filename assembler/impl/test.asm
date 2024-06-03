
include "reglib.asm"
include "archlib.asm"
include "cstring.asm"
include "io.asm"


.str
ds "hello\0"


.text

    ;!init_reglib

    vctr str
    
    call cstrlen

    !ldr1
    !println8

    loadc4 0
    exit
    

