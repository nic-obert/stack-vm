
include "archlib.asm"

.msg
ds "Hello World!\n"

.text

    loadconst8 msg
    loadconst8 13
    intrconst !PRINT_STATIC_STRING_INTR
    

