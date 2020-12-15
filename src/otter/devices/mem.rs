use std::collections::HashMap;

// The size of the normal memory is 64 kB for the text and data sections.
// Of course, the address space extends far beyond this.

#[derive(Copy, Clone)]
pub enum Size {
    Byte,
    HalfWord,
    Word,
}

// MMIO device
struct IODevice {
    size: u32,
    contents: Vec<u8>,
}

impl IODevice {
    // create a new device
    pub fn new(size: u32) -> IODevice {
        IODevice {
            size,
            contents: vec![0; size as usize],
        }
    }
}

// holds all MMIO devices; a region of memory
struct MMIO {
    addrs: Vec<u32>,
    devices: HashMap<u32, IODevice>,
}

impl MMIO {
    // make a new MMIO region
    pub fn new() -> MMIO {
        MMIO {
            addrs: Vec::new(),
            devices: HashMap::new(),
        }
    }

    // match an address to it's key in the ADDR -> DEVICE dictionary
    // e.g. if you have a device at 0x2000 of size 100
    // and a device at 0x2100 of size 100,
    // match_addr_to_key(0x2008) returns 0x2000.
    // this key can then be used to lookup the device in the dict
    fn match_addr_to_key(&self, addr: u32) -> Option<u32> {
        for key in &self.addrs {
            if addr >= *key {
                match self.devices.get(&key) {
                    None => (),
                    Some(device) => {
                        if addr < key + device.size {
                            return Some(*key);
                        }
                    }
                }
            }
        }
        None
    }

    // read the device at addr
    pub fn rd(&self, addr: u32, size: Size) -> u32 {
        let dev_addr: u32;
        // lookup the correct MMIO adress
        let device = match self.match_addr_to_key(addr) {
            None => return 0,
            Some(key) => {
                dev_addr = key;
                // lookup the device
                match self.devices.get(&key) {
                    None => return 0,
                    Some(dev) => dev,
                }
            }
        };

        // get the byte offset
        let offset = addr - dev_addr;

        let rv_size = match size {
            Size::Byte => 0,
            Size::HalfWord => 1,
            Size::Word => 2,
        };

        // read the data
        let mut data: u32 = device.contents[offset as usize] as u32;
        if rv_size >= 1 && device.size >= 2 {
            data += (device.contents[offset as usize + 1] as u32) << 8;
        }
        if rv_size >= 2 && device.size >= 4 {
            data += (device.contents[offset as usize + 2] as u32) << 16;
            data += (device.contents[offset as usize + 3] as u32) << 24;
        }
        data
    }

    // write the data to a device at addr
    pub fn wr(&mut self, addr: u32, data: u32, size: Size) {
        let dev_addr: u32;

        // see MMIO.rd for lookup process
        let device = match self.match_addr_to_key(addr) {
            None => return,
            Some(key) => {
                dev_addr = key;
                match self.devices.get_mut(&key) {
                    None => return,
                    Some(dev) => dev,
                }
            }
        };

        let offset = addr - dev_addr;

        let rv_size = match size {
            Size::Byte => 0,
            Size::HalfWord => 1,
            Size::Word => 2,
        };

        // write the data
        device.contents[offset as usize] = (data & 0x000000FF) as u8;
        if rv_size >= 1 && device.size >= 2 {
            device.contents[offset as usize + 1] = ((data & 0x0000FF00) >> 8) as u8;
        }
        if rv_size >= 2 && device.size >= 4 {
            device.contents[offset as usize + 2] = ((data & 0x00FF0000) >> 16) as u8;
            device.contents[offset as usize + 3] = ((data & 0xFF000000) >> 24) as u8;
        }
    }
}

// main memory
pub struct RAM {
    mem: Vec<Option<u8>>,
    pub size: u32,
}

impl RAM {
    // create a new main memory
    pub fn new(size: u32) -> RAM {
        RAM {
            mem: vec![None; size as usize],
            size,
        }
    }

    // try to read a byte, if unset it is read as zero
    fn try_read_byte<L>(&self, addr: u32, offset: u8, logger: L) -> u8 where L: Fn(&str) {
        let d_rd: Option<u8> = self.mem[addr as usize + offset as usize];
        match d_rd {
            None => {logger(&format!("Warning: Read unset memory at {:#010X}.", addr + (offset as u32))); 0 },
            Some(d) => d,
        }
    }

    // Reads the value at address 'addr' as an unsigned 32 bit integer.
    // 'size' is defined as: 0 for byte, 1 for half-word, or 2 for word.
    // Unused bits are not read. For example, with size=1 and data=0xFFFF, only 0x000F is returned.
    // Reading unset memory will return 0
    pub fn rd<L>(&self, addr: u32, size: Size, logger: L) -> u32 where L: Fn(&str) {
        let rv_size = match size {
            Size::Byte => 0,
            Size::HalfWord => 1,
            Size::Word => 2,
        };

        // read first byte
        let mut data = self.try_read_byte(addr, 0, &logger) as u32;

        // read second byte
        if rv_size >= 1 {
            data += (self.try_read_byte(addr, 1, &logger) as u32) << 8;
        }
        if rv_size >= 2 {
            data += (self.try_read_byte(addr, 2, &logger) as u32) << 16;
            data += (self.try_read_byte(addr, 3, &logger) as u32) << 24;
        }
        data
    }

    // write some data
    fn wr(&mut self, addr: u32, data: u32, size: Size) {
        if addr >= self.size {
            return;
        }

        let rv_size = match size {
            Size::Byte => 0,
            Size::HalfWord => 1,
            Size::Word => 2,
        };

        self.mem[addr as usize] = Some((data & 0x000000FF) as u8);
        if rv_size >= 1 {
            self.mem[addr as usize] = Some((data & 0x000000FF) as u8);
            self.mem[addr as usize + 1] = Some(((data & 0x0000FF00) >> 8) as u8);
        }
        if rv_size >= 2 {
            self.mem[addr as usize + 2] = Some(((data & 0x00FF0000) >> 16) as u8);
            self.mem[addr as usize + 3] = Some(((data & 0xFF000000) >> 24) as u8);
        }
    }
}

// a complete memory combines main memory and MMIO
pub struct Memory {
    main: RAM,
    mmio: MMIO,
    mmio_begin: u32,
}

impl Memory {
    // create a new memory
    // memory is initialized to all None (0)
    // text and stack are empty
    pub fn new(main_size: u32) -> Memory {
        Memory {
            main: RAM::new(main_size),
            mmio: MMIO::new(),
            mmio_begin: 0xFFFFFFFF,
        }
    }

    // program the memory with a binary
    pub fn prog(&mut self, binary: Vec<Vec<u8>>) {
        let binary_size = binary.len() * 4;
        if binary_size >= self.main.size as usize {
            panic!(
                "Error: Memory: {} kB binary >= {} kB",
                binary_size / 1000 - 1,
                self.main.size / 1000 - 1
            );
        }

        // loop through each word, enumerating as the word address
        for (word_addr, word) in binary.iter().enumerate() {
            // loop through each byte in a word, enumerating as the byte offset
            for (byte_offset, &byte) in word.iter().enumerate() {
                // combine the word address and byte offset as {word_addr[31:2], byte_offset[1:0]}
                // write the byte to this address at byte granularity
                self.wr(
                    ((word_addr << 2) + byte_offset) as u32,
                    byte as u32,
                    Size::Byte,
                    |_s| {}
                )
            }
        }
    }

    // map an IO device to 'addr' that contains 'size' bytes
    pub fn add_io(&mut self, addr: u32, size: u32) {
        if self.mmio.devices.contains_key(&addr) {
            eprintln!(
                "Error: Cannot map new IO device to preoccupied address {:#010X}",
                addr
            );
            return;
        }
        let device = IODevice::new(size);
        self.mmio.devices.insert(addr, device);
        self.mmio.addrs.push(addr);
        self.mmio.addrs.sort_unstable();
        self.mmio_begin = self.mmio.addrs[0];
    }

    // read from the correct region of memory
    pub fn rd<L>(&self, addr: u32, size: Size, logger: L) -> u32
        where L: Fn(&str)
    {
        if addr < self.main.size {
            self.main.rd(addr, size, logger)
        } else if addr >= self.mmio_begin {
            self.mmio.rd(addr, size)
        } else {
            logger(&format!("Error: Reading from invalid memory address {:#010X}.", addr));
            0
        }
    }

    // write to the correct region of memory
    pub fn wr<L>(&mut self, addr: u32, data: u32, size: Size, logger: L) where L: Fn(&str) {
        if addr < self.main.size {
            self.main.wr(addr, data, size);
        } else if addr >= self.mmio_begin {
            self.mmio.wr(addr, data, size);
        } else {
            logger(&format!("Error: Writing to invalid memory address {:#010X}.", addr));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn byte() {
        let mut mem = Memory::new(0x1000);
        // write then read a byte
        let addr: u32 = 0x0123;
        let data_wr: u32 = 0x000000FF;
        let data_exp: u32 = 0x000000FF;
        mem.wr(addr, data_wr, Size::Byte, |_s| {});
        let data_rd: u32 = mem.rd(addr, Size::Byte, |_s| {});
        println! {"wrote: {:#010X}, read: {:#010X}, expected: {:#010X}",
        data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn byte_overflow() {
        let mut mem = Memory::new(0x1000);
        let addr = 0x0000;
        let data_wr: u32 = 0xFFFFFFFF;
        let data_exp: u32 = 0x000000FF;
        mem.wr(addr, data_wr, Size::Byte, |_s| {});
        let data_rd: u32 = mem.rd(addr, Size::Byte, |_s| {});
        println! {"wrote: {:#010X}, read: {:#010X}, expected: {:#010X}",
        data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn halfword() {
        let mut mem = Memory::new(0x1000);
        let addr: u32 = 0x0321;
        let data_wr: u32 = 0x0000FFFF;
        let data_exp: u32 = 0x0000FFFF;
        mem.wr(addr, data_wr, Size::HalfWord, |_s| {});
        let data_rd: u32 = mem.rd(addr, Size::HalfWord, |_s| {});
        println! {"wrote: {:#010X}, read: {:#010X}, expected: {:#010X}",
        data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn halfword_overflow() {
        let mut mem = Memory::new(0x1000);
        let addr: u32 = 0x0000;
        let data_wr: u32 = 0xFFFFFFFF;
        let data_exp: u32 = 0x0000FFFF;
        mem.wr(addr, data_wr, Size::HalfWord, |_s| {});
        let data_rd: u32 = mem.rd(addr, Size::HalfWord, |_s| {});
        println! {"wrote: {:#010X}, read: {:#010X}, expected: {:#010X}",
        data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn word() {
        let mut mem = Memory::new(0x1000);
        let addr: u32 = 0x0000;
        let data_wr: u32 = 0x1234ABCD;
        let data_exp: u32 = 0x1234ABCD;
        mem.wr(addr, data_wr, Size::Word, |_s| {});
        let data_rd: u32 = mem.rd(addr, Size::Word, |_s| {});
        println! {"wrote: {:#010X}, read: {:#010X}, expected: {:#010X}",
        data_wr, data_rd, data_exp };
        assert_eq!(data_exp, data_rd);
    }

    #[test]
    fn all() {
        let mut mem = Memory::new(0x1000);
        for i in 0..mem.main.size / 4 {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0x0FFFFFFF) as u32;
            mem.wr(4 * i, data_wr, Size::Word, |_s| {});
            assert_eq!(data_wr, mem.rd(4 * i, Size::Word, |_s| {}));
        }
        for i in 0..mem.main.size / 2 {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0xFFFF) as u32;
            mem.wr(2 * i, data_wr, Size::HalfWord, |_s| {});
            assert_eq!(data_wr, mem.rd(2 * i, Size::HalfWord, |_s| {}));
        }
        for i in 0..mem.main.size {
            let data_wr: u32 = rand::thread_rng().gen_range(0, 0xFF) as u32;
            mem.wr(i, data_wr, Size::Byte, |_s| {});
            assert_eq!(data_wr, mem.rd(i, Size::Byte, |_s| {}));
        }
    }

    #[test]
    fn read_unset() {
        let mem = Memory::new(0x1000);
        mem.rd(0, Size::Word, |_s| {});
    }

    #[test]
    fn wr_invalid() {
        let mut mem = Memory::new(0x1000);
        mem.add_io(0x1000, 4);
        mem.wr(mem.main.size, 1, Size::Word, |_s| {});
        assert_eq!(0, mem.rd(mem.main.size + 40, Size::Word, |_s| {}));
    }

    #[test]
    fn rd_invalid() {
        let mut mem = Memory::new(0x1000);
        mem.add_io(0x1000, 4);
        mem.rd(mem.main.size + 40, Size::Word, |_s| {});
        assert_eq!(0, mem.rd(mem.main.size, Size::Word, |_s| {}));
    }

    #[test]
    fn prog() {
        let mut mem = Memory::new(0x1000);
        let mut binary: Vec<Vec<u8>> = vec![Vec::new(); 4];
        for i in 0..16 {
            binary[(i / 4)].push(i as u8);
        }
        mem.prog(binary);
        for i in 0..16 {
            assert_eq!(i as u32, mem.rd(i, Size::Byte, |_s| {}));
        }
    }

    #[test]
    fn simple_mmio() {
        let mut mem = Memory::new(0x1000);
        mem.add_io(0x1000, 4);
        mem.wr(0x1000, 12, Size::Byte, |_s| {});
        assert_eq!(12, mem.rd(0x1000, Size::Byte, |_s| {}));
    }
}
