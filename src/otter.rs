#[path = "./mem.rs"] pub mod mem;
#[path = "./rf.rs"] mod rf;
#[path = "./rv32i.rs"] mod rv32i;
#[path = "./file_io.rs"] mod file_io;

const MEM_SIZE: u32 = 0x10000;

const LEDS_ADDR: u32 = 0x11080000;
const LEDS_WIDTH: u32 = 2;

const SSEG_ADDR: u32 = 0x110C0000;
const SSEG_WIDTH: u32 = 2;

const SWITCHES_ADDR: u32 = 0x11000000;
const SWITCHES_WIDTH: u32 = 2;

pub struct MCU {
    pub pc: u32,
    pub mem: mem::Memory,
    pub rf: rf::RegisterFile
}

impl MCU {

    pub fn new() -> MCU {
        let mut mcu = MCU {
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

    pub fn fetch(&self) -> rv32i::Instruction {
        rv32i::decode(
            self.mem.rd(self.pc, mem::Size::Word)
        )
    }

    pub fn exec(&mut self, ir: rv32i::Instruction) {

        let rs1: u32 = self.rf.rd(ir.rs1);
        let rs2: u32 = self.rf.rd(ir.rs2);
        let mem_addr = rs1.overflowing_add(ir.imm).0;
        let jalr_target = rs1.overflowing_add(ir.imm).0;
        let branch_target = self.pc.overflowing_add(ir.imm).0;
        let jump_target = self.pc.overflowing_add(ir.imm).0;

        // note: immediates and rf reads should be cast explicitly to i32 or u32
        // operations on memories should be with unsigned integers
        // i.e. numbers are always stored/retrieved as unsigned, then interpreted/casted
        match ir.op {
            rv32i::Operation::Invalid =>
                panic!("Error: invalid instruction at {:#08x}", self.pc),

            rv32i::Operation::LUI => {
                self.rf.wr(ir.rd, ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::AUIPC => {
                self.pc += ir.imm;
                self.pc += 4;
            },

            rv32i::Operation::JAL => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = jump_target;
            },

            rv32i::Operation::JALR => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = jalr_target;
            },

            rv32i::Operation::BEQ => {
                if rs1 as i32 == rs2 as i32 {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::BNE => {
                if rs1 as i32 != rs2 as i32 {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::BLT => {
                if (rs1 as i32) < (rs2 as i32) {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::BGE => {
                if (rs1 as i32) >= (rs2 as i32) {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::BLTU => {
                if rs1 < rs2 {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::BGEU => {
                if rs1 >= rs2 {
                    self.pc = branch_target;
                } else {
                    self.pc += 4;
                }
            },

            rv32i::Operation::LB => {
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte))
            },

            rv32i::Operation::LH => {
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord));
                self.pc += 4;
            },

            rv32i::Operation::LW => {
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Word));
                self.pc += 4;
            },

            // unimplemented
            rv32i::Operation::LBU => {
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte));
                self.pc += 4;
            },

            // unimplemented
            rv32i::Operation::LHU => {
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord));
                self.pc += 4;
            },

            rv32i::Operation::SB => {
                self.mem.wr(mem_addr, rs1, mem::Size::Byte);
                self.pc += 4;
            },

            rv32i::Operation::SH => {
                self.mem.wr(mem_addr, rs2, mem::Size::HalfWord);
                self.pc += 4;
            },

            rv32i::Operation::SW => {
                self.mem.wr(mem_addr, rs2, mem::Size::Word);
                self.pc += 4;
            },

            rv32i::Operation::ADDI => {
                self.rf.wr(ir.rd, rs1 + ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::SLTI => {
                self.rf.wr(ir.rd, (rs1 < ir.imm) as u32);
                self.pc += 4;
            },

            rv32i::Operation::SLTIU => {
                self.rf.wr(ir.rd, (rs1 < ir.imm) as u32);
                self.pc += 4;
            },

            rv32i::Operation::XORI => {
                self.rf.wr(ir.rd, rs1 ^ ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::ORI => {
                self.rf.wr(ir.rd, rs1 | ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::ANDI => {
                self.rf.wr(ir.rd, rs1 & ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::SLLI => {
                self.rf.wr(ir.rd, rs1 << ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::SRLI => {
                self.rf.wr(ir.rd, rs1 >> ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::SRAI => {
                self.rf.wr(ir.rd, rs1 >> ir.imm);
                self.pc += 4;
            },

            rv32i::Operation::ADD => {
                self.rf.wr(ir.rd, rs1 + rs2);
                self.pc += 4;
            },

            rv32i::Operation::SUB => {
                self.rf.wr(ir.rd, rs1 - rs2);
                self.pc += 4;
            },

            rv32i::Operation::SLL => {
                self.rf.wr(ir.rd, rs1 << rs2);
                self.pc += 4;
            },

            rv32i::Operation::SLT => {
                self.rf.wr(ir.rd, (rs1 < rs2) as u32);
                self.pc += 4;
            },

            rv32i::Operation::SLTU => {
                self.rf.wr(ir.rd, (rs1 < rs2) as u32);
                self.pc += 4;
            },

            rv32i::Operation::XOR => {
                self.rf.wr(ir.rd, rs1 ^ rs2);
                self.pc += 4;
            },

            rv32i::Operation::SRL => {
                self.rf.wr(ir.rd, rs1 >> rs2);
                self.pc += 4;
            },

            rv32i::Operation::SRA => {
                self.rf.wr(ir.rd, rs1 >> rs2);
                self.pc += 4;
            },

            rv32i::Operation::OR => {
                self.rf.wr(ir.rd, rs1 | rs2);
                self.pc += 4;
            },

            rv32i::Operation::AND => {
                self.rf.wr(ir.rd, rs1 & rs2);
                self.pc += 4;
            }
        };
    }

    pub fn leds(&self) -> [bool; 16] {
        let mut leds = [false; 16];
        for i in 0..16 {
            //                    read a byte plus an offset          mask off the bit we care about
            //        |--------------------------------------------| |-------------------|
            leds[i] = (self.mem.rd(LEDS_ADDR + (i as u32)/8, mem::Size::Byte) & (0b1 << (i % 8))) != 0
        }
        leds
    }

    pub fn sseg(&self) -> u16 {
        self.mem.rd(SSEG_ADDR, mem::Size::HalfWord) as u16
    }

    pub fn set_sw(&mut self, index: usize, set_on: bool) {
        // cannot natively write at the bit level, so some bit mangling is required
        let prev_state = self.mem.rd(SWITCHES_ADDR, mem::Size::HalfWord);
        let updated_state: u32;
        if set_on {
            updated_state = prev_state | (0b1 << index);
        }
        else {
            updated_state = prev_state & (0b0 << index);
        }
        self.mem.wr(SWITCHES_ADDR, updated_state, mem::Size::HalfWord);
    }

    pub fn toggle_sw(&mut self, index: usize) {
        // cannot natively write at the bit level, so some bit mangling is required
        let prev_state = self.mem.rd(SWITCHES_ADDR, mem::Size::HalfWord);
        let updated_state: u32;
        updated_state = prev_state ^ (0b1 << index);
        self.mem.wr(SWITCHES_ADDR, updated_state, mem::Size::HalfWord);
    }

    pub fn switches(&self) -> [bool; 16] {
        let mut switches = [false; 16];
        for i in 0..16 {
            //                    read a byte plus an offset          mask off the bit we care about
            //        |--------------------------------------------| |-------------------|
            switches[i] = (self.mem.rd(SWITCHES_ADDR + (i as u32)/8, mem::Size::Byte) & (0b1 << (i % 8))) != 0
        }
        switches
    }

    pub fn step(&mut self) {
        self.exec(
            self.fetch()
        )
    }
}

// TODO: test each ir

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn add_addi() {
        let mut mcu = MCU::new();

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
        let mut mcu = MCU::new();

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