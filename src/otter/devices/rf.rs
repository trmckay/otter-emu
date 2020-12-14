pub const RF_SIZE: u32 = 32;

pub struct RegisterFile {
    mem: [u32; RF_SIZE as usize],
}

impl RegisterFile {
    pub fn init() -> RegisterFile {
        RegisterFile {
            mem: [0; RF_SIZE as usize],
        }
    }

    pub fn reset(&mut self) {
        for i in self.mem.iter_mut() {
            *i = 0;
        }
    }

    pub fn rd(&self, index: u32) -> u32 {
        if index < RF_SIZE && index > 0 {
            self.mem[index as usize]
        } else {
            0
        }
    }

    pub fn wr(&mut self, index: u32, data: u32) {
        if index < RF_SIZE && index > 0 {
            self.mem[index as usize] = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn all() {
        let mut rf = RegisterFile::init();
        let mut written: [u32; RF_SIZE as usize] = [0; RF_SIZE as usize];
        for i in 0..RF_SIZE {
            let rn = rand::thread_rng().gen_range(0, 0x0FFFFFFF) as u32;
            written[i as usize] = rn;
            rf.wr(i, rn);
        }
        assert_eq!(0, rf.rd(0));
        for i in 1..RF_SIZE {
            assert_eq!(written[i as usize], rf.rd(i));
        }
    }

    #[test]
    fn out_of_bounds() {
        let mut rf = RegisterFile::init();
        rf.wr(RF_SIZE, 1);
        assert_eq!(0, rf.rd(RF_SIZE));
    }
}
