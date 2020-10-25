#[path = "./bitwise.rs"] mod bitwise;

#[derive(Debug)]
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
    pub rs1: u32,
    pub rs2: u32,
    pub rd: u32,
    pub imm: u32
}

fn decode_j_imm(ir_bits: &Vec<bool>) -> u32 {
    let mut j_imm_vec = bitwise::vec_concat(&vec![ir_bits[31]; 12][..], &ir_bits[12..=19]);
    j_imm_vec = bitwise::vec_concat(&j_imm_vec[..], &vec![ir_bits[20]; 1]);
    j_imm_vec = bitwise::vec_concat(&j_imm_vec[..], &ir_bits[21..=30]);
    j_imm_vec = bitwise::vec_concat(&j_imm_vec[..], &vec![false; 1]);
    bitwise::vec_to_u32(&j_imm_vec[..])
}

fn decode_b_imm(ir_bits: &Vec<bool>) -> u32 {
    let mut b_imm_vec = bitwise::vec_concat(&vec![ir_bits[31]; 20][..], &ir_bits[7..=7]);
    b_imm_vec = bitwise::vec_concat(&b_imm_vec[..], &ir_bits[25..=30]);
    b_imm_vec = bitwise::vec_concat(&b_imm_vec[..], &ir_bits[8..=11]);
    b_imm_vec = bitwise::vec_concat(&b_imm_vec[..], &vec![false; 1]);
    bitwise::vec_to_u32(&b_imm_vec[..])
}

fn decode_u_imm(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(
        &bitwise::vec_concat(&ir_bits[12..=31], &[false; 12])[..])
}

fn decode_i_imm(ir_bits: &Vec<bool>) -> u32 {
    let i_imm_vec = bitwise::vec_concat(&vec![ir_bits[31]; 20][..], &ir_bits[20..=30]);
    bitwise::vec_to_u32(&i_imm_vec[..])
}

fn decode_s_imm(ir_bits: &Vec<bool>) -> u32 {
    let mut s_imm_vec = bitwise::vec_concat(&vec![ir_bits[31]; 20][..], &ir_bits[25..=30]);
    s_imm_vec = bitwise::vec_concat(&s_imm_vec[..], &ir_bits[7..=11]);
    bitwise::vec_to_u32(&s_imm_vec[..])
}

// TODO: turn these into macros

fn decode_funct3(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(&ir_bits[12..=14])
}

fn decode_funct7(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(&ir_bits[25..=31])
}

fn decode_rs1(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(&ir_bits[15..=19])
}

fn decode_rs2(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(&ir_bits[20..=24])
}

fn decode_rd(ir_bits: &Vec<bool>) -> u32 {
    bitwise::vec_to_u32(&ir_bits[7..=11])
}

pub fn decode(ir: u32) -> Instruction {
    let ir_bits = bitwise::u32_to_vec(ir);
    let opcode = bitwise::vec_to_u32(&ir_bits[0..=6]);

    let mut op_type = Operation::Invalid;
    let mut rd = 0;
    let mut rs1 = 0;
    let mut rs2 = 0;
    let mut imm = 0;

    // match opcode
    match opcode {
        // lui
        0b0110111 => {
            op_type = Operation::LUI;
            imm = decode_u_imm(&ir_bits);
            rd = decode_rd(&ir_bits);
        },
        // auipc
        0b0010111 => {
            op_type = Operation::AUIPC;
            imm = decode_u_imm(&ir_bits);
            rd = decode_rd(&ir_bits);
        },
        // jal
        0b1101111 => {
            op_type = Operation::JAL;
            imm = decode_j_imm(&ir_bits);
            rd = decode_rd(&ir_bits);
        },
        // jalr
        0b1100111 => {
            op_type = Operation::JALR;
            imm = decode_i_imm(&ir_bits);
            rs1 = decode_rs1(&ir_bits);
            rd = decode_rd(&ir_bits);
        },
        // branch
        0b1100011 => {
            rs1 = decode_rs1(&ir_bits);
            rs2 = decode_rs2(&ir_bits);
            imm = decode_b_imm(&ir_bits);
            match decode_funct3(&ir_bits) {
                0b000 => op_type = Operation::BEQ,
                0b001 => op_type = Operation::BNE,
                0b100 => op_type = Operation::BLT,
                0b101 => op_type = Operation::BGE,
                0b110 => op_type = Operation::BLTU,
                0b111 => op_type = Operation::BGEU,
                    _ => ()
            }
        },
        // load
        0b0000011 => {
            imm = decode_i_imm(&ir_bits);
            rs1 = decode_rs1(&ir_bits);
            rd = decode_rd(&ir_bits);
            match decode_funct3(&ir_bits) {
                0b000 => op_type = Operation::LB,
                0b001 => op_type = Operation::LH,
                0b010 => op_type = Operation::LW,
                0b100 => op_type = Operation::LBU,
                0b101 => op_type = Operation::LHU,
                    _ => ()
            }
        },
        // store
        0b0100011 => {
            imm = decode_s_imm(&ir_bits);
            rs1 = decode_rs1(&ir_bits);
            rs2 = decode_rs2(&ir_bits);
            match decode_funct3(&ir_bits) {
                0b000 => op_type = Operation::SB,
                0b001 => op_type = Operation::SH,
                0b010 => op_type = Operation::SW,
                    _ => ()
            }
        },
        // arithmetic w/ immediate
        0b0010011 => {
            imm = decode_i_imm(&ir_bits);
            rs1 = decode_rs1(&ir_bits);
            rd = decode_rd(&ir_bits);
            match decode_funct3(&ir_bits) {
                0b000 => op_type = Operation::ADDI,
                0b010 => op_type = Operation::SLTI,
                0b011 => op_type = Operation::SLTIU,
                0b100 => op_type = Operation::XORI,
                0b110 => op_type = Operation::ORI,
                0b111 => op_type = Operation::ANDI,
                0b001 => op_type = Operation::SLLI,
                0b101 => match decode_funct7(&ir_bits) {
                    0b0000000 => op_type = Operation::SRLI,
                    0b0100000 => op_type = Operation::SRAI,
                            _ => ()
                },
                    _ => ()
            }
        },
        // arithmetic
        0b0110011 => {
            rs1 = decode_rs1(&ir_bits);
            rs2 = decode_rs2(&ir_bits);
            rd = decode_rd(&ir_bits);
            match decode_funct3(&ir_bits) {
                0b000 => match decode_funct7(&ir_bits) {
                    0b0000000 => op_type = Operation::ADD,
                    0b0100000 => op_type = Operation::SUB,
                            _ => ()
                },
                0b001 => op_type = Operation::SLL,
                0b010 => op_type = Operation::SLT,
                0b011 => op_type = Operation::SLTU,
                0b100 => op_type = Operation::XOR,
                0b101 => {
                    match decode_funct7(&ir_bits) {
                        0b0000000 => op_type = Operation::SRL,
                        0b0100000 => op_type = Operation::SRA,
                                _ => ()
                    }
                }
                0b110 => op_type = Operation::OR,
                0b111 => op_type = Operation::AND,
                    _ => ()
            }
        }
        _ => ()
    }

    
    Instruction {
        op: op_type,
        rs1: rs1,
        rs2: rs2,
        rd: rd,
        imm: imm
    }
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
    fn branch1() {
        let ir_bytes: u32 = 0x01001663;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rs1, 0);
        assert_eq!(ir.rs2, 16);
        assert_eq!(ir.imm, 12);
    }

    #[test]
    fn branch2() {
        let ir_bytes: u32 = 0xfe000ae3;
        let ir = decode(ir_bytes);
        println!("got: {:#034b}", ir.imm);
        println!("exp: {:#034b}", -0xC);
        assert_eq!(-0xC, ir.imm as i32);
    }

    #[test]
    fn lui() {
        let ir_bytes: u32 = 0x0000f0b7;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 1);
        assert_eq!(ir.imm, (0b1111000000000000));
    }

    #[test]
    fn addi() {
        // addi x1, x0, 0x1d8
        let ir_bytes: u32 = 0x1d800093;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 1);
        assert_eq!(ir.rs1, 0);
        assert_eq!(ir.imm, 0x1d8);
    }

    #[test]
    fn sw() {
        let ir_bytes: u32 = 0x00112623;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rs1, 2);
        assert_eq!(ir.rs2, 1);
        assert_eq!(ir.imm, 12);
    }

    #[test]
    fn add() {
        let ir_bytes: u32 = 0x00f706b3;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 13);
        assert_eq!(ir.rs1, 14);
        assert_eq!(ir.rs2, 15);
    }

    #[test]
    fn jal() {
        // jal x31, 0x3d60
        let ir_bytes: u32 = 0x39903fef;
        let ir = decode(ir_bytes);
        println!("got: {:#034b}", ir.imm as u32);
        println!("exp: {:#034b}", 0x3d60 as u32);
        assert_eq!(ir.rd, 31);
        assert_eq!(ir.imm, 0x3d60);
    }

    #[test]
    fn jalr() {
        // jalr x1, 0x1d8
        let ir_bytes: u32 = 0x1d8100e7;
        let ir = decode(ir_bytes);
        assert_eq!(ir.rd, 1);
        assert_eq!(ir.imm, 0x1d8);
    }
}