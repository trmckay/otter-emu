const MEM_SIZE: usize = 0x1000; // in bytes

struct Memory {
    mem: [u8; MEM_SIZE],
    text_size:  usize,  // text grows up from 0x0000
    stack_size: usize  // stack grows down from 0x1000
}

impl Memory {
    // create a new memory
    // memory is initialized to all 0s
    // text and stack are empty
    pub fn init() -> Memory {
        Memory {
            mem: [0; MEM_SIZE],
            text_size:  0,
            stack_size: 0
        }
    }

    // program the memory with a binary
    pub fn prog(&mut self, binary: &[u8]) {
        // TODO: warn when binary is too large
        for (addr, &item) in binary.iter().enumerate() {
            self.mem[addr] = item;
        }
    }

    // Reads the value at address 'addr' as an unsigned 32 bit integer.
    // 'size' is defined as: 0 for byte, 1 for half-word, or 2 for word.
    // Unused bits are not read. For example, with size=1 and data=0xFFFF, only 0x000F is returned.
    // Reading unset memory will return Option::None but should be interpreted as 0
    // TODO: Undefined sizes will trigger an error.
    // TODO: Writing out of bounds will trigger an error.
    // TODO: Attempting to write to the text section will trigger a warning.
    pub fn rd(&self, addr: usize, size: usize) -> u32 {
        // read byte
        let mut data: u32 = self.mem[addr] as u32;
        // shift an read in extra byte
        if size >= 1 {
            data <<= 4;
            data += self.mem[addr + 1] as u32;
        }
        // shift and read in extra 2 bytes
        if size >= 2 {
            data <<= 8;
            data += ((self.mem[addr + 2] as u32) <<  4) + (self.mem[addr + 3] as u32)
        }
        data
    }

    // Writes the value in 'data' to address 'addr'.
    // 'size' is defined as: 0 for byte, 1 for half-word, or 2 for word.
    // Unused bits are not written. For example, with size=1 and data=0xFFFF,
    // only 0xF is written to the byte at 'addr'.
    // TODO: Undefined sizes will trigger an error.
    // TODO: Writing out of bounds will trigger an error.
    // TODO: Attempting to write to the text section will trigger a warning.
    pub fn wr(&mut self, addr: usize, data: u32, size: usize) {
        // write byte
        self.mem[addr] = data as u8;
        // write remaining byte for half-word
        if size >= 1 {
            self.mem[addr + 1] = (data >> 4) as u8;
        }
        // write remaining bytes for word
        if size >= 2 {
            self.mem[addr + 2] = (data >> 8)  as u8;
            self.mem[addr + 3] = (data >> 12) as u8;
        }
    }
}