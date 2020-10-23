#[path = "./mem.rs"] mod mem;
#[path = "./rf.rs"] mod rf;
#[path = "./rv32i.rs"] mod rv32i;
#[path = "./file_io.rs"] mod file_io;

const MEM_SIZE: usize = 0x1000;

const LEDS_ADDR: usize = 0x11080000;
const LEDS_WIDTH: usize = 4;

const SSEG_ADDR: usize = 0x110C0000;
const SSEG_WIDTH: usize = 2;

const SWITCHES_ADDR: usize = 0x11000000;
const SWITCHES_WIDTH: usize = 4;

pub struct Otter {
    pc: usize,
    mem: mem::Memory,
    rf: rf::RegisterFile
}

impl Otter {

    pub fn new() -> Otter {
        let mut mcu = Otter {
            pc: 0,
            mem: mem::Memory::new(MEM_SIZE),
            rf: rf::RegisterFile::init(),
        };

        // map IO
        mcu.mem.add_io(LEDS_ADDR, LEDS_WIDTH);
        mcu.mem.add_io(SSEG_ADDR, SSEG_WIDTH);
        mcu.mem.add_io(SWITCHES_ADDR, SWITCHES_WIDTH);

        mcu
    }

    // Loads a binary from the path "binary" into the main memory.
    // Text section begins at zero. Binary should not exceed 64 kB.
    pub fn load_bin(&mut self, binary: &str) {
        println!("Reading binary file {}...", binary);
        self.mem.prog(
            file_io::file_to_bytes(binary)
        );
    }

    fn fetch(&self) -> rv32i::Instruction {
        rv32i::decode(
            self.mem.rd(self.pc, mem::Size::Word)
        )
    }

    fn exec(&mut self, ir: rv32i::Instruction) {

        let rs1_signed: i32 = self.rf.rd(ir.rs1) as i32;
        let rs1_unsigned: u32 = self.rf.rd(ir.rs1) as u32;
        let rs2_signed: i32 = self.rf.rd(ir.rs2) as i32;
        let rs2_unsigned: u32 = self.rf.rd(ir.rs2) as u32;

        // addr is an unsigned (always positive) 32 bit integer +/- a signed immediate
        // accomplished by masking out the MSB of the offset then interpreting it as signed,
        // adding the offset, then converting back to unsigned, and restoring the MSB
        let mem_addr: usize =
            (((rs1_unsigned & 0xEFFFFFFF) as i32 
            + ir.imm as i32) as u32
            + (rs1_unsigned & 0x80000000)) as usize;

        // note: immediates and rf reads should be cast explicitly to i32 or u32
        // operations on memories should be with unsigned integers
        // i.e. numbers are always stored/retrieved as unsigned, then interpreted/casted
        match ir.op {
            rv32i::Operation::Invalid =>
                panic!("Error: invalid instruction at {:#08x}", self.pc),

            rv32i::Operation::LUI =>
                self.rf.wr(ir.rd, ir.imm),

            rv32i::Operation::AUIPC =>
                self.pc = self.pc + (ir.imm as i32) as usize,

            rv32i::Operation::JAL => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = self.pc + (ir.imm as i32) as usize
            },

            rv32i::Operation::JALR => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = (rs1_signed + ir.imm as i32) as usize;
            },

            rv32i::Operation::BEQ => {
                if rs1_signed ==  rs1_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::BNE => {
                if rs1_signed != rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::BLT => {
                if rs1_signed < rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::BGE => {
                if rs1_signed >= rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::BLTU => {
                if rs1_unsigned <= rs2_unsigned {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::BGEU => {
                if rs1_unsigned >= rs2_unsigned {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i::Operation::LB =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte)),

            rv32i::Operation::LH =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord)),

            rv32i::Operation::LW =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Word)),

            // unimplemented
            rv32i::Operation::LBU =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte)),

            // unimplemented
            rv32i::Operation::LHU =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord)),

            rv32i::Operation::SB => {
                self.mem.wr(mem_addr, rs1_unsigned, mem::Size::Byte);
            },

            rv32i::Operation::SH => {
                self.mem.wr(mem_addr, rs2_unsigned, mem::Size::HalfWord);
            },

            rv32i::Operation::SW => {
                self.mem.wr(mem_addr, rs2_unsigned, mem::Size::Word);
            },

            rv32i::Operation::ADDI => {
                self.rf.wr(ir.rd, (rs1_signed + (ir.imm as i32)) as u32);
            },

            rv32i::Operation::SLTI => {
                self.rf.wr(ir.rd, (rs1_signed < (ir.imm as i32)) as u32);
            },

            rv32i::Operation::SLTIU => {
                self.rf.wr(ir.rd, (rs1_unsigned < (ir.imm as u32)) as u32);
            },

            rv32i::Operation::XORI => {
                self.rf.wr(ir.rd, rs1_unsigned ^ (ir.imm as u32));
            },

            rv32i::Operation::ORI => {
                self.rf.wr(ir.rd, rs1_unsigned | (ir.imm as u32));
            },

            rv32i::Operation::ANDI => {
                self.rf.wr(ir.rd, rs1_unsigned & (ir.imm as u32));
            },

            rv32i::Operation::SLLI => {
                self.rf.wr(ir.rd, (rs1_unsigned << (ir.imm as i32)) as u32);
            },

            rv32i::Operation::SRLI => {
                self.rf.wr(ir.rd, (rs1_unsigned >> (ir.imm as i32)) as u32);
            },

            rv32i::Operation::SRAI => {
                self.rf.wr(ir.rd, (rs1_signed >> (ir.imm as i32)) as u32);
            },

            rv32i::Operation::ADD => {
                self.rf.wr(ir.rd, (rs1_signed + rs2_signed) as u32)
            },

            rv32i::Operation::SUB => {
                self.rf.wr(ir.rd, (rs1_signed - rs2_signed) as u32)
            },

            rv32i::Operation::SLL => {
                self.rf.wr(ir.rd, (rs1_unsigned << rs2_signed) as u32)
            },

            rv32i::Operation::SLT => {
                self.rf.wr(ir.rd, (rs1_signed < rs2_signed) as u32)
            },

            rv32i::Operation::SLTU => {
                self.rf.wr(ir.rd, (rs1_unsigned < rs2_unsigned) as u32)
            },

            rv32i::Operation::XOR => {
                self.rf.wr(ir.rd, (rs1_signed ^ rs2_signed) as u32)
            },

            rv32i::Operation::SRL => {
                self.rf.wr(ir.rd, (rs1_unsigned >> rs2_signed) as u32)
            },

            rv32i::Operation::SRA => {
                self.rf.wr(ir.rd, (rs1_signed >> rs2_signed) as u32)
            },

            rv32i::Operation::OR =>
            {
                self.rf.wr(ir.rd, (rs1_signed | rs2_signed) as u32)
            },

            rv32i::Operation::AND => {
                self.rf.wr(ir.rd, (rs1_signed & rs2_signed) as u32)

            }
        };
    }

    pub fn run(&mut self) {
        println!("\n Initialized successfully! Beginning execution.\n");
        loop {
            self.exec(
                self.fetch()
            );
        }
    }
}

// TODO: test each ir

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn simple_add() {
        let mut mcu = Otter::new();

        //addi x1, x0, 2
        //addi x2, x0, 3
        mcu.exec(rv32i::Instruction {
            op: rv32i::Operation::ADDI,
            rs1: 0, rs2: 0, rd: 1, imm: 2
        });
        assert_eq!(2, mcu.rf.rd(1));

        mcu.exec(rv32i::Instruction {
            op: rv32i::Operation::ADDI,
            rs1: 0, rs2: 0, rd: 2, imm: 3
        });

        assert_eq!(2, mcu.rf.rd(1));
        assert_eq!(3, mcu.rf.rd(2));

        // add x3, x1, x2
        mcu.exec(rv32i::Instruction {
            op: rv32i::Operation::ADD,
            rs1: 1, rs2: 2, rd: 3, imm: 0
        });

        assert_eq!(2, mcu.rf.rd(1));
        assert_eq!(3, mcu.rf.rd(2));
        assert_eq!(5, mcu.rf.rd(3));
    }

    #[test]
    fn multi_add() {
        let mut mcu = Otter::new();

        let mut total = 0;
        for _i in 0..32 {
            let operand: u32 = rand::thread_rng().gen_range(0, 0xFF) as u32;
            mcu.exec(rv32i::Instruction {
                op: rv32i::Operation::ADDI,
                rs1: 0, rs2: 0, rd: 2, imm: operand
            });
            mcu.exec(rv32i::Instruction {
                op: rv32i::Operation::ADD,
                rs1: 1, rs2: 2, rd: 1, imm: 0
            });
            total += operand;
        }
        assert_eq!(total, mcu.rf.rd(1));
    }
}