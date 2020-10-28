# otter-emu

WIP emulator for Cal Poly's RISC-V RV32I chip written in Rust

## Preview

Example output in debug mode:
```
+----------------------------------------------------+
| PC: 0x0000021C
+----------------------------------------------------+
|      Instruction: ADDI (0xFFF00793)
|  Operand 1 (rs1): x0 = 0 / 0x0
|  Operand 2 (rs2): x0 = 0 / 0x0
| Destination (rd): 15
|        Immediate: 4294967295 / 0xffffffff
+----------------------------------------------------+
|   Register |      Signed |    Unsigned |        Hex
|------------|-------------|-------------|------------
|  x0, zero  |           0 |           0 |        0x0
|  x1, ra    |           0 |           0 |        0x0
|  x2, sp    |      -32768 |  4294934528 | 0xFFFF8000
|  x3, gp    |           6 |           6 |        0x6
|  x4, tp    |           0 |           0 |        0x0
|  x5, t0    |           0 |           0 |        0x0
|  x6, t1    |           0 |           0 |        0x0
|  x7, t2    |           0 |           0 |        0x0
|  x8, s0    |           0 |           0 |        0x0
|  x9, s1    |           0 |           0 |        0x0
| x10, a0    |          15 |          15 |        0xF
| x11, a1    |   285999104 |   285999104 | 0x110C0000
| x12, a2    |   285736960 |   285736960 | 0x11080000
| x13, a3    |           0 |           0 |        0x0
| x14, a4    |           8 |           8 |        0x8
| x15, a5    |           0 |           0 |        0x0
| x16, a6    |   285212672 |   285212672 | 0x11000000
| x17, a7    |           0 |           0 |        0x0
| x18, s2    |           0 |           0 |        0x0
| x19, s3    |           0 |           0 |        0x0
| x20, s4    |           0 |           0 |        0x0
| x21, s5    |           0 |           0 |        0x0
| x22, s6    |           0 |           0 |        0x0
| x23, s7    |           0 |           0 |        0x0
| x24, s8    |           0 |           0 |        0x0
| x25, s9    |           0 |           0 |        0x0
| x26, s10   |           0 |           0 |        0x0
| x27, s11   |           0 |           0 |        0x0
| x28, t3    |           0 |           0 |        0x0
| x29, t4    |           0 |           0 |        0x0
| x30, t5    |           1 |           1 |        0x1
| x31, t6    |         124 |         124 |       0x7C
+----------------------------------------------------+
|            F E D C B A 9 8 7 6 5 4 3 2 1 0
|      LEDs: - - - - - - - - - - - - * * * * 
|  Switches: - - - - - - - - - - - - - - - - 
| 7-Segment: 0x0008
+----------------------------------------------------+

Hit breakpoint 0x0000021C
Press enter to step
```

## Build requirements

- rust

## Build

`cargo build`

## Run tests

`cargo test`
