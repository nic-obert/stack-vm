
include "archlib.asm"

.hi
ds "Hi\n"

@print_hi
    loadconst8 hi
    loadconst8 3
    intrconst !PRINT_STATIC_STRING_INTR

