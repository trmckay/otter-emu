#[path = "./mem.rs"] mod mem;
#[path = "./rf.rs"] mod rf;
#[path = "./rv32i_ir.rs"] mod rv32i_ir;
#[path = "./file_io.rs"] mod file_io;

pub struct Otter {
    pc: usize,
    mem: mem::Memory,
    rf: rf::RegisterFile,
    leds: Vec<bool>,
    sseg: u16,
    switches: Vec<bool>
}

impl Otter {

    pub fn init(binary: &str) -> Otter {
        let mut mcu = Otter {
            pc: 0,
            mem: mem::Memory::init(),
            rf: rf::RegisterFile::init(),
            leds: vec![false; 16],
            sseg: 0,
            switches: vec![false; 16]
        };
        mcu.load_bin(binary);
        mcu
    }

    // Loads a binary from the path "binary" into the main memory.
    // Text section begins at zero. Binary should not exceed 64 kB.
    fn load_bin(&mut self, binary: &str) {
        println!("Reading binary file {}...", binary);
        self.mem.prog(
            file_io::file_to_bytes(binary)
        );
    }

    fn _fetch(&self) -> rv32i_ir::Instruction {
        rv32i_ir::decode(
            self.mem.rd(self.pc, mem::Size::Word)
        )
    }

    // todo: MMIO
    fn _exec(&mut self, ir: rv32i_ir::Instruction) {

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
            rv32i_ir::Operation::Invalid =>
                panic!("Error: invalid instruction at {:#08x}", self.pc),

            rv32i_ir::Operation::LUI =>
                self.rf.wr(ir.rd, ir.imm),

            rv32i_ir::Operation::AUIPC =>
                self.pc = self.pc + (ir.imm as i32) as usize,

            rv32i_ir::Operation::JAL => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = self.pc + (ir.imm as i32) as usize
            },

            rv32i_ir::Operation::JALR => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = (rs1_signed + ir.imm as i32) as usize;
            },

            rv32i_ir::Operation::BEQ => {
                if rs1_signed ==  rs1_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::BNE => {
                if rs1_signed != rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::BLT => {
                if rs1_signed < rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::BGE => {
                if rs1_signed >= rs2_signed {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::BLTU => {
                if rs1_unsigned <= rs2_unsigned {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::BGEU => {
                if rs1_unsigned >= rs2_unsigned {
                    self.pc = self.pc + (ir.imm as i32) as usize;
                }
            },

            rv32i_ir::Operation::LB =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte)),

            rv32i_ir::Operation::LH =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord)),

            rv32i_ir::Operation::LW =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Word)),

            // unimplemented
            rv32i_ir::Operation::LBU =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::Byte)),

            // unimplemented
            rv32i_ir::Operation::LHU =>
                self.rf.wr(ir.rd, self.mem.rd(mem_addr, mem::Size::HalfWord)),

            rv32i_ir::Operation::SB => {
                self.mem.wr(mem_addr, rs1_unsigned, mem::Size::Byte);
            },

            rv32i_ir::Operation::SH => {
                self.mem.wr(mem_addr, rs2_unsigned, mem::Size::HalfWord);
            },

            rv32i_ir::Operation::SW => {
                self.mem.wr(mem_addr, rs2_unsigned, mem::Size::Word);
            },

            rv32i_ir::Operation::ADDI => {
                self.rf.wr(ir.rd, (rs1_signed + (ir.imm as i32)) as u32);
            },

            rv32i_ir::Operation::SLTI => {
                self.rf.wr(ir.rd, (rs1_signed < (ir.imm as i32)) as u32);
            },

            rv32i_ir::Operation::SLTIU => {
                self.rf.wr(ir.rd, (rs1_unsigned < (ir.imm as u32)) as u32);
            },

            rv32i_ir::Operation::XORI => {
                self.rf.wr(ir.rd, rs1_unsigned ^ (ir.imm as u32));
            },

            rv32i_ir::Operation::ORI => {
                self.rf.wr(ir.rd, rs1_unsigned | (ir.imm as u32));
            },

            rv32i_ir::Operation::ANDI => {
                self.rf.wr(ir.rd, rs1_unsigned & (ir.imm as u32));
            },

            rv32i_ir::Operation::SLLI => {
                self.rf.wr(ir.rd, (rs1_unsigned << (ir.imm as i32)) as u32);
            },

            rv32i_ir::Operation::SRLI => {
                self.rf.wr(ir.rd, (rs1_unsigned >> (ir.imm as i32)) as u32);
            },

            rv32i_ir::Operation::SRAI => {
                self.rf.wr(ir.rd, (rs1_signed >> (ir.imm as i32)) as u32);
            },

            rv32i_ir::Operation::ADD => {
                self.rf.wr(ir.rd, (rs1_signed + rs2_signed) as u32)
            },

            rv32i_ir::Operation::SUB => {
                self.rf.wr(ir.rd, (rs1_signed - rs2_signed) as u32)
            },

            rv32i_ir::Operation::SLL => {
                self.rf.wr(ir.rd, (rs1_unsigned << rs2_signed) as u32)
            },

            rv32i_ir::Operation::SLT => {
                self.rf.wr(ir.rd, (rs1_signed < rs2_signed) as u32)
            },

            rv32i_ir::Operation::SLTU => {
                self.rf.wr(ir.rd, (rs1_unsigned < rs2_unsigned) as u32)
            },

            rv32i_ir::Operation::XOR => {
                self.rf.wr(ir.rd, (rs1_signed ^ rs2_signed) as u32)
            },

            rv32i_ir::Operation::SRL => {
                self.rf.wr(ir.rd, (rs1_unsigned >> rs2_signed) as u32)
            },

            rv32i_ir::Operation::SRA => {
                self.rf.wr(ir.rd, (rs1_signed >> rs2_signed) as u32)
            },

            rv32i_ir::Operation::OR => {
                self.rf.wr(ir.rd, (rs1_signed | rs2_signed) as u32)
            },

            rv32i_ir::Operation::AND => {
                self.rf.wr(ir.rd, (rs1_signed & rs2_signed) as u32)

            }
        };
    }

    pub fn run(&mut self) {
        println!("\n Initialized successfully! Beginning execution.\n");
        loop {
            self._exec(
                self._fetch()
            );
        }
    }
}

// TODO: test each ir

#[cfg(test)]
mod test {

}