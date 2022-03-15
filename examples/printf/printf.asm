SYSFN_EXIT  = 0
SYSFN_READ  = 1
SYSFN_WRITE = 2

CHAR_LF = 10

MEMORY_HI = 0x4000

    mem     MEMORY_HI

    seg     0x2000
fmt.lower_hex_digits: d8 "0123456789abcdef"
fmt.upper_hex_digits: d8 "0123456789ABCDEF"

str1: d8 "Строка: '%s', десятичное число: %d, hex lowercase: %x, hex uppercase: %X, процент: %%.", CHAR_LF, 0
str2: d8 "моя строка", 0
str3: d8 "Много параметров: %X %X %X %X %X %X %X %X %X %X %X %X %X %X %X %X.", CHAR_LF, 0

    seg     0x1000
entry:
    li      %sp, MEMORY_HI

    li      %a0, str1
    li      %a1, str2
    li      %a2, 1234
    lui     %a3, 0xDEADB
    addi    %a3, %a3, 0xEEF
    lui     %a4, 0xCAFEB
    addi    %a4, %a4, 0xABE
    call    printf

    addi    %sp, %sp, -(11*4)
    li      %a0, str3
    li      %a1, 0
    li      %a2, 1
    li      %a3, 2
    li      %a4, 3
    li      %a5, 4
    li      %s0, 5
    st      %s0, %sp, 0*4
    li      %s0, 6
    st      %s0, %sp, 1*4
    li      %s0, 7
    st      %s0, %sp, 2*4
    li      %s0, 8
    st      %s0, %sp, 3*4
    li      %s0, 9
    st      %s0, %sp, 4*4
    li      %s0, 10
    st      %s0, %sp, 5*4
    li      %s0, 11
    st      %s0, %sp, 6*4
    li      %s0, 12
    st      %s0, %sp, 7*4
    li      %s0, 13
    st      %s0, %sp, 8*4
    li      %s0, 14
    st      %s0, %sp, 9*4
    li      %s0, 15
    st      %s0, %sp, 10*4
    call    printf

    sysfn   %zero, SYSFN_EXIT

printf:
    addi    %sp, %sp, -(11*4)
    st      %s0, %sp, 0*4
    st      %s1, %sp, 1*4
    st      %s2, %sp, 2*4
    st      %s3, %sp, 3*4
    st      %s4, %sp, 4*4
    st      %s5, %sp, 5*4
    st      %a1, %sp, 6*4
    st      %a2, %sp, 7*4
    st      %a3, %sp, 8*4
    st      %a4, %sp, 9*4
    st      %a5, %sp, 10*4
    addi    %a1, %sp, 6*4
    li      %s0, '%'
    li      %s1, 's'
    li      %s2, 'd'
    li      %s3, 'x'
    li      %s4, 'X'

.printf.loop:
    ld.u8   %a2, %a0, 0
    beq     %a2, %zero, .printf.exit
.printf.loop.nonzero:
    ld.u8   %a3, %a0, 1
    beq     %a2, %s0, .printf.fmt
    addi    %a0, %a0, 1
    sysfn   %a2, SYSFN_WRITE
    mov     %a2, %a3
    bne     %a2, %zero, .printf.loop.nonzero
.printf.exit:
    ld      %s0, %sp, 0*4
    ld      %s1, %sp, 1*4
    ld      %s2, %sp, 2*4
    ld      %s3, %sp, 3*4
    ld      %s4, %sp, 4*4
    ld      %s5, %sp, 5*4
    addi    %sp, %sp, 11*4
    ret

.printf.fmt:
    addi    %a0, %a0, 2
    beq     %a3, %s1, .printf.fmt.str
    beq     %a3, %s2, .printf.fmt.dec
    beq     %a3, %s3, .printf.fmt.hex.lower
    beq     %a3, %s4, .printf.fmt.hex.upper
    beq     %a3, %s0, .printf.fmt.percent

; Invalid format spec. Print format string as-is.
    sysfn   %a2, SYSFN_WRITE
    beq     %a3, %zero, .printf.exit
.printf.fmt.percent:
    sysfn   %a3, SYSFN_WRITE
    jmp     .printf.loop

.printf.fmt.str:
    ld      %a2, %a1, 0
    addi    %a1, %a1, 4
    ld.u8   %a3, %a2, 0
    beq     %a3, %zero, .printf.loop
.printf.fmt.str.loop:
    sysfn   %a3, SYSFN_WRITE
    ld.u8   %a3, %a2, 1
    addi    %a2, %a2, 1
    bne     %a3, %zero, .printf.fmt.str.loop
    jmp     .printf.loop

.printf.fmt.hex.upper:
    li      %a5, fmt.upper_hex_digits
    jmp     .printf.fmt.hex.generic
.printf.fmt.hex.lower:
    li      %a5, fmt.lower_hex_digits
.printf.fmt.hex.generic:
    ld      %a4, %a1, 0
    addi    %a1, %a1, 4
    addi    %a2, %sp, -1
    st.u8   %zero, %a2, 0
.printf.fmt.hex.loop:
    ; r = num & 0x1F
    ; *--s = table[r]
    ; num = num >> 4
    ; if num != 0 repeat
    andi    %a3, %a4, 0xF
    add     %a3, %a3, %a5
    ld.u8   %a3, %a3, 0
    st.u8   %a3, %a2, -1
    addi    %a2, %a2, -1
    lshri   %a4, %a4, 4
    bne     %a4, %zero, .printf.fmt.hex.loop
    jmp     .printf.fmt.str.loop

.printf.fmt.dec:
    lui     %a5, 0xCCCCC
    addi    %a5, %a5, 0xCCD
    ld      %a4, %a1, 0
    addi    %a1, %a1, 4
    addi    %a2, %sp, -1
    st.u8   %zero, %a2, 0
.printf.fmt.dec.loop:
    ; q = upper32(num * 0xCCCCCCCD) >> 3
    ; r = num - q * 10
    ; *--s = r + '0'
    ; num = q
    ; if num != 0 repeat
    mulwu   %zero, %s5, %a4, %a5
    lshri   %s5, %s5, 3
    muli    %a3, %s5, 10
    sub     %a3, %a4, %a3
    addi    %a3, %a3, '0'
    st.u8   %a3, %a2, -1
    addi    %a2, %a2, -1
    mov     %a4, %s5
    bne     %a4, %zero, .printf.fmt.dec.loop
    jmp     .printf.fmt.str.loop
