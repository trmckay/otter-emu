use super::super::util::*;
use super::devices::mem;
use super::devices::rf;
use super::rv32i::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub const MEM_SIZE: usize = 0x10000;
pub const RF_SIZE: usize = 32;

const LEDS_ADDR: u32 = 0x11080000;
const LEDS_WIDTH: u32 = 2;

const SSEG_ADDR: u32 = 0x110C0000;
const SSEG_WIDTH: u32 = 2;

const SWITCHES_ADDR: u32 = 0x11000000;
const SWITCHES_WIDTH: u32 = 2;

pub struct MCU {
    pub pc: u32,
    mem: mem::Memory,
    rf: rf::RegisterFile,
}

impl MCU {
    pub fn new() -> MCU {
        let mut mcu = MCU {
            pc: 0,
            mem: mem::Memory::new(MEM_SIZE as u32),
            rf: rf::RegisterFile::init(),
        };

        // map IO
        mcu.mem.add_io(LEDS_ADDR, LEDS_WIDTH);
        mcu.mem.add_io(SSEG_ADDR, SSEG_WIDTH);
        mcu.mem.add_io(SWITCHES_ADDR, SWITCHES_WIDTH);

        mcu
    }

    #[allow(dead_code)]
    pub fn from_bin(binary: &str) -> MCU {
        let mut mcu = MCU::new();
        mcu.load_bin(binary);
        mcu
    }

    // Loads a binary from the path "binary" into the main memory.
    // Text section begins at zero. Binary should not exceed 64 kB.
    pub fn load_bin(&mut self, binary: &str) {
        self.mem.prog(io::file_to_bytes(binary));
    }

    pub fn dump<L>(&self, path: &str, logger: L) where L: Fn(&str) {
        let path = Path::new(path);
        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => { logger(&format!("Error: Could not open file {}: {}.", path.display(), why)); return },
            Ok(f) => f
        };

        // dump register file
        if let Err(why) = file.write_all(b"REGISTER FILE CONTENTS:\n") {
            logger(&format!("Error: Could write to file {}: {}.", path.display(), why));
            return;
        };
        for i in 0..RF_SIZE {
            let left = format!("x{} ({}):", i, decode::reg_name(i as u32));
            let right = format!("{:#010X}", self.rf_rd(i as u32));
            let line = format!("    {:10} {}\n", left, right);
            if let Err(why) = file.write_all(line.as_bytes()) {
                logger(&format!("Error: Could write to file {}: {}.", path.display(), why));
                return;
            };
        }

        // dump memory
        if let Err(why) = file.write_all(b"\nMEMORY CONTENTS:\n") {
            logger(&format!("Error: Could write to file {}: {}.", path.display(), why));
            return;
        };
        for i in 0..MEM_SIZE/4 {
            let addr = i * 4;
            let b0 = self.mem_rd(addr as u32, mem::Size::Byte);
            let b1 = self.mem_rd((addr + 1) as u32, mem::Size::Byte);
            let b2 = self.mem_rd((addr + 2) as u32, mem::Size::Byte);
            let b3 = self.mem_rd((addr + 3) as u32, mem::Size::Byte);
            let line = format!("    {:08x?}: {:04x?} {:04x?} {:04x?} {:04x?}\n", addr, b3, b2, b1, b0);
            if let Err(why) = file.write_all(line.as_bytes()) {
                logger(&format!("Error: Could write to file {}: {}.", path.display(), why));
                return;
            }
        }
        logger(&format!("Dumped state to {}.", path.display()));
    }

    // step once; closure defines logging method
    pub fn step<L>(&mut self, logger: L)
    where
        L: Fn(&str),
    {
        let ir = self.fetch(|s| logger(s));
        let ir = MCU::validate(ir.0, self.pc, |s| logger(s));
        self.exec(ir, |s| logger(s));
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.rf.reset();
    }

    // dump the register file
    pub fn rf(&self) -> Vec<u32> {
        let mut rf_dump = vec![0; rf::RF_SIZE as usize];
        for (i, d) in rf_dump.iter_mut().enumerate() {
            *d = self.rf.rd(i as u32);
        }
        rf_dump
    }

    #[allow(dead_code)]
    pub fn rf_rd(&self, addr: u32) -> u32 {
        self.rf.rd(addr)
    }

    pub fn mem_rd(&self, addr: u32, size: mem::Size) -> u32 {
        self.mem.rd(addr, size, |_s| {})
    }

    fn incr_pc(&mut self) {
        self.pc = self.pc.overflowing_add(4).0;
    }

    // validates the instruction, logging errors, returns a fixed instruction
    pub fn validate<L>(ir: decode::Instruction, pc: u32, logger: L) -> decode::Instruction
    where
        L: Fn(&str),
    {
        //    - check mem read value on LOAD
        //    - check jump/branch target within text
        let nop = decode::Instruction {
            op: decode::Operation::ADD,
            rs1: 0,
            rs2: 0,
            rd: 0,
            imm: 0,
        };

        // check for invalid instruction
        if let decode::Operation::Invalid = ir.op {
            logger(&format!(
                "[{:#010X}] Error: Invalid instruction.",
                pc
            ));
            return nop;
        };

        // check for read/write non-existent register
        if ir.rd > 31 || ir.rs1 > 31 || ir.rs2 > 31 {
            logger(&format!(
                "[{:#010X}] Error: Access to a non-existent register.",
                pc
            ));
            return nop;
        }
        ir
    }

    pub fn fetch<L>(&self, logger: L) -> (decode::Instruction, u32)
    where
        L: Fn(&str),
    {
        let ir = self.mem.rd(self.pc, mem::Size::Word, logger);
        (decode::decode(ir), ir)
    }

    fn exec<L>(&mut self, ir: decode::Instruction, logger: L)
    where
        L: Fn(&str),
    {
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
            decode::Operation::Invalid => {
                logger(&format!("[{:#010X}] Error: Instruction was corrupted. This is an error in the emulator, not the program.", self.pc));
            }

            decode::Operation::LUI => {
                self.rf.wr(ir.rd, ir.imm);
                self.incr_pc();
            }

            decode::Operation::AUIPC => {
                self.rf.wr(ir.rd, self.pc.overflowing_add(ir.imm).0);
                self.incr_pc();
            }

            decode::Operation::JAL => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = jump_target;
            }

            decode::Operation::JALR => {
                self.rf.wr(ir.rd, self.pc as u32 + 4);
                self.pc = jalr_target;
            }

            decode::Operation::BEQ => {
                if rs1 as i32 == rs2 as i32 {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::BNE => {
                if rs1 as i32 != rs2 as i32 {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::BLT => {
                if (rs1 as i32) < (rs2 as i32) {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::BGE => {
                if (rs1 as i32) >= (rs2 as i32) {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::BLTU => {
                if rs1 < rs2 {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::BGEU => {
                if rs1 >= rs2 {
                    self.pc = branch_target;
                } else {
                    self.incr_pc();
                }
            }

            decode::Operation::LB => {
                let mut byte = self
                    .mem
                    .rd(mem_addr.overflowing_add(ir.imm).0, mem::Size::Byte, logger);
                // sign extend
                if byte & 0b10000000 != 0 {
                    byte |= 0xFFFFFF00;
                }
                self.rf.wr(ir.rd, byte);
                self.incr_pc();
            }

            decode::Operation::LH => {
                let mut halfword: u32 = self
                    .mem
                    .rd(mem_addr.overflowing_add(ir.imm).0, mem::Size::HalfWord, logger);
                // sign extend
                if halfword & 0b1000000000000000 != 0 {
                    halfword |= 0xFFFF0000;
                }
                self.rf.wr(ir.rd, halfword);
                self.incr_pc();
            }

            decode::Operation::LW => {
                self.rf.wr(
                    ir.rd,
                    self.mem
                        .rd(mem_addr.overflowing_add(ir.imm).0, mem::Size::Word, logger),
                );
                self.incr_pc();
            }

            // unimplemented
            decode::Operation::LBU => {
                self.rf.wr(
                    ir.rd,
                    self.mem
                        .rd(mem_addr.overflowing_add(ir.imm).0, mem::Size::Byte, logger),
                );
                self.incr_pc();
            }

            // unimplemented
            decode::Operation::LHU => {
                self.rf.wr(
                    ir.rd,
                    self.mem
                        .rd(mem_addr.overflowing_add(ir.imm).0, mem::Size::HalfWord, logger),
                );
                self.incr_pc();
            }

            decode::Operation::SB => {
                self.mem
                    .wr(mem_addr.overflowing_add(ir.imm).0, rs1, mem::Size::Byte, logger);
                self.incr_pc();
            }

            decode::Operation::SH => {
                self.mem
                    .wr(mem_addr.overflowing_add(ir.imm).0, rs2, mem::Size::HalfWord, logger);
                self.incr_pc();
            }

            decode::Operation::SW => {
                self.mem
                    .wr(mem_addr.overflowing_add(ir.imm).0, rs2, mem::Size::Word, logger);
                self.incr_pc();
            }

            decode::Operation::ADDI => {
                self.rf.wr(ir.rd, rs1.overflowing_add(ir.imm).0);
                self.incr_pc();
            }

            decode::Operation::SLTI => {
                self.rf.wr(ir.rd, ((rs1 as i32) < (ir.imm as i32)) as u32);
                self.incr_pc();
            }

            decode::Operation::SLTIU => {
                self.rf.wr(ir.rd, (rs1 < ir.imm) as u32);
                self.incr_pc();
            }

            decode::Operation::XORI => {
                self.rf.wr(ir.rd, rs1 ^ ir.imm);
                self.incr_pc();
            }

            decode::Operation::ORI => {
                self.rf.wr(ir.rd, rs1 | ir.imm);
                self.incr_pc();
            }

            decode::Operation::ANDI => {
                self.rf.wr(ir.rd, rs1 & ir.imm);
                self.incr_pc();
            }

            decode::Operation::SLLI => {
                self.rf.wr(ir.rd, rs1.overflowing_shl(ir.imm).0);
                self.incr_pc();
            }

            decode::Operation::SRLI => {
                self.rf.wr(ir.rd, rs1.overflowing_shr(ir.imm).0);
                self.incr_pc();
            }

            decode::Operation::SRAI => {
                self.rf
                    .wr(ir.rd, (rs1 as i32).overflowing_shr(ir.imm).0 as u32);
                self.incr_pc();
            }

            decode::Operation::ADD => {
                self.rf.wr(ir.rd, rs1.overflowing_add(rs2).0);
                self.incr_pc();
            }

            decode::Operation::SUB => {
                self.rf.wr(ir.rd, rs1.overflowing_sub(rs2).0);
                self.incr_pc();
            }

            decode::Operation::SLL => {
                self.rf
                    .wr(ir.rd, (rs1 as i32).overflowing_shl(rs2).0 as u32);
                self.incr_pc();
            }

            decode::Operation::SLT => {
                self.rf.wr(ir.rd, ((rs1 as i32) < (rs2 as i32)) as u32);
                self.incr_pc();
            }

            decode::Operation::SLTU => {
                self.rf.wr(ir.rd, (rs1 < rs2) as u32);
                self.incr_pc();
            }

            decode::Operation::XOR => {
                self.rf.wr(ir.rd, rs1 ^ rs2);
                self.incr_pc();
            }

            decode::Operation::SRL => {
                self.rf.wr(ir.rd, rs1.overflowing_shr(rs2).0);
                self.incr_pc();
            }

            decode::Operation::SRA => {
                self.rf
                    .wr(ir.rd, (rs1 as i32).overflowing_shr(rs2).0 as u32);
                self.incr_pc();
            }

            decode::Operation::OR => {
                self.rf.wr(ir.rd, rs1 | rs2);
                self.incr_pc();
            }

            decode::Operation::AND => {
                self.rf.wr(ir.rd, rs1 & rs2);
                self.incr_pc();
            }
        };
    }

    pub fn leds(&self) -> Vec<bool> {
        let mut leds = vec![false; 16];
        for (i, l) in leds.iter_mut().enumerate() {
            //                    read a byte plus an offset          mask off the bit we care about
            //        |--------------------------------------------| |-------------------|
            *l = (self.mem_rd(LEDS_ADDR + (i as u32) / 8, mem::Size::Byte) & (0b1 << (i % 8))) != 0
        }
        leds
    }

    pub fn sseg(&self) -> u16 {
        self.mem_rd(SSEG_ADDR, mem::Size::HalfWord) as u16
    }

    pub fn toggle_sw(&mut self, index: usize) {
        let prev_state = self.mem_rd(SWITCHES_ADDR, mem::Size::HalfWord);
        let updated_state: u32;
        updated_state = prev_state ^ (0b1 << index);
        self.mem
            .wr(SWITCHES_ADDR, updated_state, mem::Size::HalfWord, |_s| {});
    }
}

// TODO: test each ir

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_all() {
        // FAILING
        // - 0x14, 20 SW
        // - 0x1F, 31 LH
        // - 0x20, 32 LB
        // - 0x23, 35 SH
        // - 0x24, 36 SB
        let mut do_break = false;
        let mut mcu = MCU::new();
        mcu.load_bin("res/programs/test/all/bin");
        loop {
            // first test
            if mcu.pc == 0x18 {
                if do_break {
                    break;
                } else {
                    do_break = true;
                }
            }
            mcu.exec(mcu.fetch(|_s| {}).0, |_s| {});
            // if ssegs are 0xffff, test-all fails
            assert!(mcu.sseg() != 0xFFFF);
        }
    }

    #[test]
    fn add_addi() {
        let mut mcu = MCU::new();

        //addi x1, x0, 2
        //addi x2, x0, 3
        mcu.exec(
            decode::Instruction {
                op: decode::Operation::ADDI,
                rs1: 0,
                rs2: 0,
                rd: 1,
                imm: 2,
            },
            |_s| {},
        );
        assert_eq!(2, mcu.rf.rd(1));

        mcu.exec(
            decode::Instruction {
                op: decode::Operation::ADDI,
                rs1: 0,
                rs2: 0,
                rd: 2,
                imm: 3,
            },
            |_s| {},
        );

        assert_eq!(2, mcu.rf.rd(1));
        assert_eq!(3, mcu.rf.rd(2));

        // add x3, x1, x2
        mcu.exec(
            decode::Instruction {
                op: decode::Operation::ADD,
                rs1: 1,
                rs2: 2,
                rd: 3,
                imm: 0,
            },
            |_s| {},
        );

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
            mcu.exec(
                decode::Instruction {
                    op: decode::Operation::ADDI,
                    rs1: 0,
                    rs2: 0,
                    rd: 2,
                    imm: operand,
                },
                |_s| {},
            );
            mcu.exec(
                decode::Instruction {
                    op: decode::Operation::ADD,
                    rs1: 1,
                    rs2: 2,
                    rd: 1,
                    imm: 0,
                },
                |_s| {},
            );
            total += operand;
        }
        assert_eq!(total, mcu.rf.rd(1));
    }

    #[test]
    fn stepping() {
        let mut mcu = MCU::new();
        mcu.load_bin("./res/programs/test/all/bin");
        mcu.step(|_s| {});
    }
}
