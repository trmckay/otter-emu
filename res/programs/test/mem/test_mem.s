init:
    li s1, 0xF00
    li s2, 0x110C0000 # SSEG
    li s3, 0x11080000 # LEDs

loop:

    test_word:
        addi t0, t0, 1
        beq t0, x0, test_halfword
        sw t0, 0(s1)
        lw t1, 0(s1)
        sw t0, 0(s3)
        beq t0, t1, test_word

        call fail

    test_halfword:
        li t2, 0x10000
        addi t0, t0, 1
        beq t0, t2, test_byte
        sh t0, 0(s1)
        lh t1, 0(s1)
        beq t0, t1, test_halfword
        call fail

    test_byte:
        li t2, 0x100
        addi t0, t0, 1
        beq t0, t2, done
        sb t0, 0(s1)
        lb t1, 0(s1)
        beq t0, t1, test_byte
        call fail

    j loop

fail:
    addi s0, s0, 1
    sw s0, 0(s2)
    ret

done:
    li t0, 0xFFFF
    sw t0, 0(s3)
    j done
