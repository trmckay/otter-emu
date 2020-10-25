// takes a slice of a boolean vector and returns the
// unsigned integer representation of those bits
// returns a u128, but can be casted down if safe to do so
pub fn vec_to_u32(bits: &[bool]) -> u32 {
    let mut total: u64 = 0;
    for i in 0..bits.len() {
        if bits[i] {
            total += (0b1) << i;
        }
    }
    total as u32
}

// takes a number and converts it to a vector of booleans
// that represent its bits
// * in order to remain consistent with the indices, bits will have their endian-ness reversed
// * i.e. in the integer, the LSB or bit 0 exists on the right,
// * but in vec form, the LSB or bit 0 exists on the left so it can be in index 0
pub fn u32_to_vec(num: u32) -> Vec<bool> {
    let mut bits: Vec<bool> = vec![false; 32];
    for i in 0..32 {
        bits[i] = (num & (0b1 << i)) != 0;
    }
    bits
}

// takes two vectors and returns a new vector that concatenates the second to the first
pub fn vec_concat(first: &[bool], sec: &[bool]) -> Vec<bool> {
    let mut r = Vec::new();
    r.extend_from_slice(sec);
    r.extend_from_slice(first);
    r
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn vec_cvt() {
        for i in 0..0xFFFF {
            let vec = u32_to_vec(i);
            assert_eq!(i, vec_to_u32(&vec));
        }
    }

    #[test]
    fn vec_cat() {
        let vec1 = vec![true, true, false, true];
        let vec2 = vec![true, false, false, false];
        let r = vec_concat(&vec1[..], &vec2[0..2]);
        assert_eq!(vec![true, false, true, true, false, true], r);
    }

    #[test]
    fn num_ops() {
        let n1 = 0x2CF8;
        let bits = u32_to_vec(n1);
        let r1 = vec_concat(&bits[0..4], &bits[8..12]);
        assert_eq!(vec![false, false, true, true, false, false, false, true], r1);
        assert_eq!(0x8C, vec_to_u32(&r1));
    }
}