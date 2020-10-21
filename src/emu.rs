#[path = "./mem.rs"] mod mem;
#[path = "./rf.rs"] mod rf;
#[path = "./rv32i_ir.rs"] mod rv32i_ir;
#[path = "./file_io.rs"] mod file_io;

pub struct otter {
    pc: usize,
    mem: mem::Memory,
    rf: rf::RegisterFile,
}

impl otter {

    pub fn run(&mut self, binary: &str) {
        self.load_bin(binary);
        loop {
            self.exec(self.fetch())
        }
    }

    // Returns the number of seconds a real would take to run "binary"
    // at a clock speed of "clk_mHz."
    pub fn bench(&mut self, binary: &str, stop_pc: usize, clk_mHz: u32) -> f32 {
        self.load_bin(binary);
        let mut n_cycles: u32 = 0;
        while self.pc != stop_pc {
            self.exec(self.fetch());
            n_cycles += 2;
        }
        (n_cycles as f32)/(10E6 * clk_mHz as f32)
    }

    // Loads a binary from the path "binary" into the main memory.
    // Text section begins at zero. Binary should not exceed 64 kB.
    fn load_bin(&mut self, binary: &str) {

    }

    fn fetch(&self) -> rv32i_ir::Instruction {
        rv32i_ir::decode(self.mem.rd(self.pc, mem::Size::Word))
    }

    fn exec(&mut self, ir: rv32i_ir::Instruction) {
    }
}