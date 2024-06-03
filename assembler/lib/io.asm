
include "archlib.asm"


%print8
    
    intrconst !PRINT8_INTR

%endmacro


%println8

    intrconst !PRINT8_INTR
    loadc1 '\n'
    intrconst !PRINT_CHAR_INTR

%endmacro

