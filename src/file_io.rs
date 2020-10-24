use std::fs::File;
use std::io::Read;

// adapted from Mara Bos at:
// https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor
// returns a vector of words, where each word is a vector of four bytes
pub fn file_to_bytes(path: &str) -> Vec<Vec<u8>> {
    let mut file = match File::open(String::from(path)) {
        Err(why) => panic!("Error: could not open {}: {}", path, why),
        Ok(f) => f,
    };

    let mut list_of_chunks = Vec::new();

    let chunk_size = 0x4;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = match file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk) {
            Err(why) => panic!("Error: could not read file {}: {}", path, why),
            Ok(r) => r
        };
        if n == 0 { break; }
        list_of_chunks.push(chunk);
        if n < chunk_size { break; }
    }
    list_of_chunks
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bin_io() {
        let bytes = file_to_bytes("res/programs/test_all/test_all.bin");
        // first word
        assert_eq!(0x37, bytes[0][0]);
        assert_eq!(0x08, bytes[0][1]);
        assert_eq!(0x00, bytes[0][2]);
        assert_eq!(0x11, bytes[0][3]);
        // second word
        assert_eq!(0xB7, bytes[1][0]);
        assert_eq!(0x05, bytes[1][1]);
        assert_eq!(0x0C, bytes[1][2]);
        assert_eq!(0x11, bytes[1][3]);
        // last few byes
        assert_eq!(0xEF, bytes[0x3FAC >> 2][0]);
        assert_eq!(0xBE, bytes[0x3FAC >> 2][1]);
        assert_eq!(0x00, bytes[0x3FAC >> 2][2]);
        assert_eq!(0x00, bytes[0x3FAC >> 2][3]);
    }
}