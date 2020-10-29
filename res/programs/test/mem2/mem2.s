# memtest2

init:
    li  s0, 0xF00
    li  s1, 0xF10

byte:
    li  t0, -4
    sb  t0,  8(s0)
    lb  t1, -8(s1)
    lbu t2,  8(s0)

halfword:
    li  t0,  -3
    sh  t0,  -8(s1)
    lh  t1,   8(s0)
    lhu t2,  -8(s1)

word:
    li  t0,  -1
    sw  t0,  -8(s1)
    lw  t1,   8(s0)
    lw t2,   -8(s1)

j byte
