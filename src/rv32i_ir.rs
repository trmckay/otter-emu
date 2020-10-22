use std::num::Wrapping;

pub enum Operation {
    // load upper
    LUI, AUIPC,
    // jump
    JAL, JALR,
    // branch
    BEQ, BNE, BLT, BGE, BLTU, BGEU,
    // load
    LB, LH, LW, LBU, LHU,
    // store
    SB, SH, SW,
    // arithmetic w/ immediates
    ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI,
    // arithmetic
    ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND,
    // no match
    Invalid
}

pub struct Instruction {
    pub op: Operation,
    pub rs1: usize,
    pub rs2: usize,
    pub rd: usize,
    pub imm: u32
}

// avert your eyes from this monstrosity
pub fn decode(ir_bytes: u32) -> Instruction {

    // decoded values
    let opcode: u32 = ir_bytes & 0b00000000000000000000000001111111;
    let funct3: u32 = (ir_bytes & 0b00000000000000000111000000000000) >> 12;
    let funct7: u32 = (ir_bytes & 0b11111110000000000000000000000000) >> 25;

    // i_imm = {20{ir_bytes[32]}, ir_bytes[30:20]}
    let mut i_imm = Wrapping(0u32);
    if (ir_bytes & 0b1000000000000000000000000000) > 0 {
        i_imm += Wrapping(0b11111111111111111111000000000000u32);
    }
    i_imm += Wrapping((ir_bytes & 0b01111111111100000000000000000000u32) >> 20 as u32);

    // s_imm = {20{ir_bytes[31]}, ir_bytes[30, 25], ir_bytes[11:7]}
    let mut s_imm = Wrapping(0u32);
    if ir_bytes & ir_bytes & 0b1000000000000000000000000000 > 0 {
        s_imm += Wrapping(0b11111111111111111111000000000000u32);
    }
    s_imm += Wrapping((ir_bytes & 0b01111110000000000000000000000000) >> 20 as u32);
    s_imm += Wrapping((ir_bytes & 0b00000000000000000000111110000000) >> 7 as u32);

    // u_imm = {ir_bytes[31:12], 12{0}}
    let u_imm = Wrapping(ir_bytes & 0b11111111111111111111000000000000 as u32);

    // b_imm = {12{ir_bytes[31]}, ir_bytes[7], ir_bytes[30:25], ir_bytes[11:8], 0}
    let mut b_imm = Wrapping(0u32);
    if ir_bytes & 0b1000000000000000000000000000 > 0 {
        b_imm += Wrapping(0b11111111111111111111000000000000 as u32);
    }
    b_imm += Wrapping((ir_bytes & 0b00000000000000000000000010000000) << 8 as u32);
    b_imm += Wrapping((ir_bytes & 0b01111110000000000000000000000000) >> 20 as u32);
    b_imm += Wrapping((ir_bytes & 0b00000000000000000000111100000000) >> 7 as u32);

    // j_imm = {12{ir_bytes[31]}, ir_bytes[19:12], ir_bytes[20], ir_bytes[30:21], 0}
    let mut j_imm = Wrapping(0u32);
    if ir_bytes & 0b1000000000000000000000000000 > 0 {
        j_imm += Wrapping(0b11111111111100000000000000000000u32);
    }
    j_imm += Wrapping(ir_bytes & 0b00000000000011111111000000000000 as u32);
    j_imm += Wrapping((ir_bytes & 0b00000000000100000000000000000000) >> 9 as u32);
    j_imm += Wrapping((ir_bytes & 0b01111111111000000000000000000000) >> 20 as u32);

    let mut ir = Instruction {
        op: Operation::Invalid,
        rs1: ((ir_bytes & 0b00000000000011111000000000000000) >> 15) as usize,
        rs2: ((ir_bytes & 0b00000001111100000000000000000000) >> 20) as usize,
        rd:  ((ir_bytes & 0b00000000000000000000111110000000) >> 7) as usize,
        imm: 0
    };

    // match opcode
    match opcode {
        0b0110111 => { ir.op = Operation::LUI; ir.imm = u_imm.0},
        0b0010111 => { ir.op = Operation::AUIPC; ir.imm = u_imm.0},
        0b1101111 => { ir.op = Operation::JAL; ir.imm = j_imm.0},
        0b1100111 => { ir.op = Operation::JALR; ir.imm = i_imm.0},
        // branch
        0b1100011 => {
            ir.imm = b_imm.0;
            match funct3 {
                0b000 => ir.op = Operation::BEQ,
                0b001 => ir.op = Operation::BNE,
                0b100 => ir.op = Operation::BLT,
                0b101 => ir.op = Operation::BGE,
                0b110 => ir.op = Operation::BLTU,
                0b111 => ir.op = Operation::BGEU,
                    _ => ir.op = Operation::Invalid
            }
        },
        // load
        0b0000011 => {
            ir.imm = i_imm.0;
            match funct3 {
                0b000 => ir.op = Operation::LB,
                0b001 => ir.op = Operation::LH,
                0b010 => ir.op = Operation::LW,
                0b100 => ir.op = Operation::LBU,
                0b101 => ir.op = Operation::LHU,
                    _ => ir.op = Operation::Invalid
            }
        },
        // store
        0b0100011 => {
            ir.imm = s_imm.0;
            match funct3 {
                0b000 => ir.op = Operation::SB,
                0b001 => ir.op = Operation::SH,
                0b101 => ir.op = Operation::SW,
                    _ => ir.op = Operation::Invalid
            }
        },
        // arithmetic w/ immediate
        0b0010011 => {
            ir.imm = i_imm.0;
            match funct3 {
                0b000 => ir.op = Operation::ADDI,
                0b010 => ir.op = Operation::SLTI,
                0b011 => ir.op = Operation::SLTIU,
                0b100 => ir.op = Operation::XORI,
                0b110 => ir.op = Operation::ORI,
                0b111 => ir.op = Operation::ANDI,
                0b001 => ir.op = Operation::SLLI,
                0b101 => match funct7 {
                    0b0000000 => ir.op = Operation::SRLI,
                    0b0100000 => ir.op = Operation::SRAI,
                            _ => ir.op = Operation::Invalid
                },
                    _ => ir.op = Operation::Invalid
            }
        },
        // arithmetic
        0b0110011 => match funct3 {
                0b000 => match funct7 {
                    0b0000000 => ir.op = Operation::ADD,
                    0b0100000 => ir.op = Operation::SUB,
                            _ => ir.op = Operation::Invalid
                },
                0b001 => ir.op = Operation::SLL,
                0b010 => ir.op = Operation::SLT,
                0b011 => ir.op = Operation::SLTU,
                0b100 => ir.op = Operation::XOR,
                0b101 => {
                    match funct7 {
                        0b0000000 => ir.op = Operation::SRL,
                        0b0100000 => ir.op = Operation::SRA,
                                _ => ir.op = Operation::Invalid
                    }
                }
                0b110 => ir.op = Operation::OR,
                0b111 => ir.op = Operation::AND,
                    _ => ir.op = Operation::Invalid
        }
        _ => ir.op = Operation::Invalid
    }
    ir
}

// TODO: this needs more testing; ideally 1-2 tests per instruction
#[cfg(test)]
mod tests {
    use super::*;

    /*
    0x01001663: bne x0, x16, 12
    0x0000f0b7: lui x1, 15
    0x07c18f93: addi x31, x3, 124
    0x00112623: sw x1, 12(x2)
    0x00f706b3: add x13, x14, x15
    0x000000ef: jal x1, 0
    */

    #[test]
    fn test_branch() {
        let ir_bytes: u32 = 0x01001663;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rs1, 0);
        assert_eq!(ir.rs2, 16);
        assert_eq!(ir.imm, 12);
    }

    #[test]
    fn test_upper() {
        let ir_bytes: u32 = 0x0000f0b7;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rs1, 1);
        assert_eq!(ir.imm, (0b1111000000000000));
    }

    #[test]
    fn test_arithm_imm() {
        let ir_bytes: u32 = 0x07c18f93;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 31);
        assert_eq!(ir.rs1, 3);
        assert_eq!(ir.imm, 124);
    }

    #[test]
    fn test_store() {
        let ir_bytes: u32 = 0x00112623;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rs1, 2);
        assert_eq!(ir.rs2, 1);
        assert_eq!(ir.imm, 12);
    }

    #[test]
    fn test_arithm() {
        let ir_bytes: u32 = 0x00f706b3;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 13);
        assert_eq!(ir.rs1, 14);
        assert_eq!(ir.rs2, 15);
    }

    #[test]
    fn test_jump() {
        let ir_bytes: u32 = 0x000000ef;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 1);
        assert_eq!(ir.imm, 0);
    }
}