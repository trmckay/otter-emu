const MEM_SIZE: usize = 0x1000; // in bytes

pub enum Size {
    Byte,
    HalfWord,
    Word,
}

pub struct Memory {
    mem: [Option<u8>; MEM_SIZE],
    text_size:  usize,  // text grows up from 0x0000
    stack_size: usize   // stack grows down from 0x1000
}

impl Memory {
    // create a new memory
    // memory is initialized to all None (0)
    // text and stack are empty
    pub fn init() -> Memory {
        Memory {
            mem: [None; MEM_SIZE],
            text_size:  0,
            stack_size: 0
        }
    }

    // program the memory with a binary
    pub fn prog(&mut self, binary: &[u8]) {
        assert!(binary.len() <= MEM_SIZE);
        for (addr, &item) in binary.iter().enumerate() {
            self.mem[addr] = Some(item);
        }
        self.text_size = binary.len();
    }

    // Reads the value at address 'addr' as an unsigned 32 bit integer.
    // 'size' is defined as: 0 for byte, 1 for half-word, or 2 for word.
    // Unused bits are not read. For example, with size=1 and data=0xFFFF, only 0x000F is returned.
    // Reading unset memory will return 0 and generate a warning
    pub fn rd(&self, addr: usize, size: Size) -> u32 {

        if addr >= MEM_SIZE {
            eprintln!("Error: reading out of bounds memory ({:#08x})", addr);
            return 0
        }

        let rv_size;
        match size {
            Size::Byte => {
                rv_size = 0;
            },
            Size::HalfWord => {
                rv_size = 1;
                if addr % 2 != 0 {
                    eprintln!("Warning: not reading on half-word boundary ({:#08x})", addr);
                }
            },
            Size::Word => {
                rv_size = 2;
                if addr % 4 != 0 {
                    eprintln!("Warning: not reading on word boundary ({:#08x})", addr);
                }
            }
        }

        fn _try_read_byte(mem: [Option<u8>; MEM_SIZE], addr: usize, offset: usize) -> u8 {
            let d_rd: Option<u8> = mem[addr + offset];
            match d_rd {
                None => {
                    eprintln!("Warning: reading unset byte as zero ({:#08x})", addr + offset);
                    0
                },
                Some(d) => d
            }
        }

        // read first byte
        let mut data = _try_read_byte(self.mem, addr, 0) as u32;

        // read second byte
        if rv_size >= 1 {
            data += (_try_read_byte(self.mem, addr, 1) as u32) << 8;
        }
        if rv_size >= 2 {
            data += (_try_read_byte(self.mem, addr, 2) as u32) << 16;
            data += (_try_read_byte(self.mem, addr, 3) as u32) << 24;
        }
        data
    }

    // Writes the value in 'data' to address 'addr'.
    // 'size' is defined as: 0 for byte, 1 for half-word, or 2 for word.
    // Unused bits are not written. For example, with size=1 and data=0xFFFF,
    // only 0xF is written to the byte at 'addr'.
    pub fn wr(&mut self, addr: usize, data: u32, size: Size) {

        if addr >= MEM_SIZE {
            eprintln!("Warning: attempting to write to an out of bounds memory location ({:#08x})",
                addr);
            return
        }

        if addr < self.text_size {
            eprintln!("Warning: writing to the text section ({:#08x})", addr)
        }

        let rv_size = match size {
            Size::Byte => 0,
            Size::HalfWord => 1,
            Size::Word => 2
        };

        self.mem[addr] = Some((data & 0x000000FF) as u8);
        if rv_size >= 1 {
            self.mem[addr] = Some((data & 0x000000FF) as u8);
            self.mem[addr + 1] = Some(((data & 0x0000FF00) >> 8) as u8);
        }
        if rv_size >= 2 {
            self.mem[addr + 2] = Some(((data & 0x00FF0000) >> 16) as u8);
            self.mem[addr + 3] = Some(((data & 0xFF000000) >> 24) as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_init() {
        let mem = Memory::init();
        assert_eq!(0, mem.stack_size);
        assert_eq!(0, mem.text_size);
        assert_eq!(MEM_SIZE, mem.mem.len());
    }

    #[test]
    fn test_byte() {
        let mut mem = Memory::init();
        // write then read a byte
        let addr: usize = 0x0123;
        let data_wr: u32 = 0x000000FF;
        let data_exp: u32 = 0x000000FF;
        mem.wr(addr, data_wr, Size::Byte);
        let data_rd: u32 = mem.rd(addr, Size::Byte);
        println!{"wrote: {:#08x}, read: {:#08x}, expected: {:#08x}",
            data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn test_byte_overflow() {
        let mut mem = Memory::init();
        let addr = 0x0000;
        let data_wr: u32 = 0xFFFFFFFF;
        let data_exp: u32 = 0x000000FF;
        mem.wr(addr, data_wr, Size::Byte);
        let data_rd: u32 = mem.rd(addr, Size::Byte);
        println!{"wrote: {:#08x}, read: {:#08x}, expected: {:#08x}",
            data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn test_halfword() {
        let mut mem = Memory::init();
        let addr: usize = 0x0321;
        let data_wr: u32  = 0x0000FFFF;
        let data_exp: u32 = 0x0000FFFF;
        mem.wr(addr, data_wr, Size::HalfWord);
        let data_rd: u32 = mem.rd(addr, Size::HalfWord);
        println!{"wrote: {:#08x}, read: {:#08x}, expected: {:#08x}",
            data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn test_halfword_overflow() {
        let mut mem = Memory::init();
        let addr: usize = 0x0000;
        let data_wr: u32  = 0xFFFFFFFF;
        let data_exp: u32 = 0x0000FFFF;
        mem.wr(addr, data_wr, Size::HalfWord);
        let data_rd: u32 = mem.rd(addr, Size::HalfWord);
        println!{"wrote: {:#08x}, read: {:#08x}, expected: {:#08x}",
            data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn test_word() {
        let mut mem = Memory::init();
        let addr: usize = 0x0000;
        let data_wr: u32  = 0x1234ABCD;
        let data_exp: u32 = 0x1234ABCD;
        mem.wr(addr, data_wr, Size::Word);
        let data_rd: u32 = mem.rd(addr, Size::Word);
        println!{"wrote: {:#08x}, read: {:#08x}, expected: {:#08x}",
            data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn test_all() {
        let mut mem = Memory::init();
        for i in 0..MEM_SIZE/4 {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0x0FFFFFFF) as u32;
            mem.wr(4*i, data_wr, Size::Word);
            assert_eq!(data_wr, mem.rd(4*i, Size::Word));
        }
        for i in 0..MEM_SIZE/2 {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0xFFFF) as u32;
            mem.wr(2*i, data_wr, Size::HalfWord);
            assert_eq!(data_wr, mem.rd(2*i, Size::HalfWord));
        }
        for i in 0..MEM_SIZE {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0xFF) as u32;
            mem.wr(i, data_wr, Size::Byte);
            assert_eq!(data_wr, mem.rd(i, Size::Byte));
        }
    }

    #[test]
    fn test_read_unset() {
        let mem = Memory::init();
        mem.rd(0, Size::Word);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut mem = Memory::init();
        mem.wr(MEM_SIZE, 1, Size::Word);
        assert_eq!(0, mem.rd(MEM_SIZE, Size::Word));
    }

    #[test]
    fn test_prog() {
        let mut mem = Memory::init();
        let mut binary: [u8; 16] = [0; 16];
        for i in 0..16 {
            binary[i as usize] = i as u8;
        }
        mem.prog(&binary);
        assert_eq!(16, mem.text_size);
        for i in 0..16 {
            assert_eq!(i as u32, mem.rd(i, Size::Byte));
        }
    }
}