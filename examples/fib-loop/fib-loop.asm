SYSFN_EXIT  = 0
SYSFN_READ  = 1
SYSFN_WRITE = 2

CHAR_LF = 10

MEMORY_HI = 0x4000

    mem     MEMORY_HI

    seg     0x2000
welcome_msg: d8 "Программа для вычисления числа Фибоначчи.", CHAR_LF, "Введите число: ", 0
result_msg:  d8 "Результат: ", 0

    seg     0x1000
entry:
    li      %sp, MEMORY_HI
    li      %a0, welcome_msg
    call    print_str
    call    scan_uint
    call    fib
    mov     %s0, %a0
    li      %a0, result_msg
    call    print_str
    mov     %a0, %s0
    call    print_uint
    li      %a0, CHAR_LF
    sysfn   %a0, SYSFN_WRITE
    sysfn   %zero, SYSFN_EXIT

fib:
    li      %a1, 0
    li      %a2, 1
    beq     %a0, %zero, .fib.exit
.fib.loop:
    add     %a3, %a1, %a2
    mov     %a1, %a2
    mov     %a2, %a3
    addi    %a0, %a0, -1
    bne     %a0, %zero, .fib.loop
.fib.exit:
    mov     %a0, %a1
    ret

print_str:
    ld.u8   %a1, %a0, 0
    beq     %a1, %zero, .print_str.exit
.print_str.loop:
    sysfn   %a1, SYSFN_WRITE
    ld.u8   %a1, %a0, 1
    addi    %a0, %a0, 1
    bne     %a1, %zero, .print_str.loop
.print_str.exit:
    ret

print_uint:
    lui     %a4, 0xCCCCC
    addi    %a4, %a4, 0xCCD
    mov     %a2, %a0
    addi    %a0, %sp, -1
    st.u8   %zero, %a0, 0
.print_uint.loop:
    ; q = upper32(num * 0xCCCCCCCD) >> 3
    ; r = num - q * 10
    ; *--s = r + '0'
    ; num = q
    ; if num != 0 repeat
    mulwu   %zero, %a3, %a2, %a4
    lshri   %a3, %a3, 3
    muli    %a1, %a3, 10
    sub     %a1, %a2, %a1
    addi    %a1, %a1, '0'
    st.u8   %a1, %a0, -1
    addi    %a0, %a0, -1
    mov     %a2, %a3
    bne     %a2, %zero, .print_uint.loop
    jmp     .print_str.loop

scan_uint:
    li      %a0, 0
    li      %a1, 9
.scan_uint.loop:
    sysfn   %a2, SYSFN_READ
    addi    %a2, %a2, -'0'
    bgtu    %a2, %a1, .scan_uint.exit
    muli    %a0, %a0, 10
    add     %a0, %a0, %a2
    jmp     .scan_uint.loop
.scan_uint.exit:
    ret
