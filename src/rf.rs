const RF_SIZE: u32 = 32;

pub struct RegisterFile {
    mem: [u32; RF_SIZE as usize],
}

impl RegisterFile<> {

    pub fn init<>() -> RegisterFile<> {
        RegisterFile {
            mem: [0; RF_SIZE as usize]
        }
    }

    pub fn rd(&self, index: u32) -> u32 {
        if index >= RF_SIZE {
            eprintln!("Warning: register {} does not exist", index);
            return 0
        }
        self.mem[index as usize]
    }

    pub fn wr(&mut self, index: u32, data: u32) {
        if index >= RF_SIZE {
            eprintln!("Warning: register {} does not exist", index);
            return
        }
        if index == 0 {
            eprintln!("Warning: attempting to write to x0")
        }
        else {
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