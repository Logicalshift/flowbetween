use flo_float_encoder::*;

use std::time::{Duration};

///
/// Character encoding for a 6-bit value
///
pub (super) const ENCODING_CHAR_SET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'
];

///
/// Represents a target for serialized animation data
///
pub trait AnimationDataTarget {
    ///
    /// Writes a single character to this target
    ///
    fn write_chr(&mut self, chr: char);

    ///
    /// Encodes a byte array to this target
    ///
    fn write_bytes(&mut self, bytes: &[u8]) {
        let mut bit         = 0;
        let mut remainder   = 0u8;

        for b in bytes {
            // Write the first character from this byte
            let num_bits    = 6-bit;
            let mask        = (1<<num_bits)-1;
            let val         = remainder | ((b&mask)<<bit);

            self.write_chr(ENCODING_CHAR_SET[val as usize]);

            // Work out how many remaining bits
            let bits_left   = 8 - num_bits;
            if bits_left == 6 {
                // Write a second character and reset to the beginning
                self.write_chr(ENCODING_CHAR_SET[(b>>2) as usize]);
                bit         = 0;
                remainder   = 0;
            } else {
                // Remainder of this byte is combined with the next character
                remainder   = b >> num_bits;
                bit         = bits_left; 
            }
        }

        // Write the final character
        if bit > 0 {
            self.write_chr(ENCODING_CHAR_SET[remainder as usize]);
        }
    }

    ///
    /// Writes a usize
    ///
    fn write_usize(&mut self, data: usize) {
        // These have a UTF-8 like format where we write them 6 bits at a time and the top bit indicates if there are more
        // bits to come: this is much more efficient for small sizes
        let mut remaining = data;
        loop {
            let lower_bits  = remaining & 0x1f;
            remaining       = remaining >> 5;
            let to_write    = if remaining > 0 { lower_bits | 0x20 } else { lower_bits };

            self.write_chr(ENCODING_CHAR_SET[to_write as usize]);
            if remaining == 0 { break; }
        }
    }

    ///
    /// Writes a u64 that's expected to have a small value
    ///
    fn write_small_u64(&mut self, data: u64) {
        // These have a UTF-8 like format where we write them 6 bits at a time and the top bit indicates if there are more
        // bits to come: this is much more efficient for small sizes
        let mut remaining = data;
        loop {
            let lower_bits  = remaining & 0x1f;
            remaining       = remaining >> 5;
            let to_write    = if remaining > 0 { lower_bits | 0x20 } else { lower_bits };

            self.write_chr(ENCODING_CHAR_SET[to_write as usize]);
            if remaining == 0 { break; }
        }
    }

    ///
    /// Writes a string value to this target
    ///
    fn write_str(&mut self, data: &str) {
        let utf8 = data.as_bytes();

        self.write_usize(utf8.len());
        self.write_bytes(utf8);
    }

    ///
    /// Writes a u32 value to this target
    ///
    fn write_u32(&mut self, data: u32) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a i32 value to this target
    ///
    fn write_i32(&mut self, data: i32) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a u64 value to this target
    ///
    fn write_u64(&mut self, data: u64) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a u64 value to this target
    ///
    fn write_i64(&mut self, data: i64) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a f32 value to this target
    ///
    fn write_f32(&mut self, data: f32) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a f64 value to this target
    ///
    fn write_f64(&mut self, data: f64) {
        self.write_bytes(&data.to_le_bytes());
    }

    ///
    /// Writes a f64 value to this target that's relative to a previous value (this uses a more compact format to save space)
    ///
    fn write_next_f64(&mut self, last: f64, data: f64) {
        let mut bytes  = vec![];
        squish_float(&mut bytes, last, data).ok();
        self.write_bytes(&bytes);
    }

    ///
    /// Writes a time to the target
    ///
    fn write_duration(&mut self, duration: Duration) {
        // Time in microseconds
        let micros = duration.as_micros() as u64;

        // Can fit up to around 4000s (66 minutes) in a u32
        if micros <= 0xffffffff {
            self.write_chr('t');
            self.write_u32(micros as u32);
        } else {
            self.write_chr('T');
            self.write_u64(micros);
        }
    }
}

impl AnimationDataTarget for String {
    fn write_chr(&mut self, chr: char) {
        self.push(chr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64;

    #[test]
    fn encode_u8_0() {
        let mut res = String::new();
        res.write_bytes(&[0u8]);
        assert!(&res == "AA");
    }

    #[test]
    fn encode_u8_255_once() {
        let mut res = String::new();
        res.write_bytes(&[255u8]);
        assert!(&res == "/D");
    }

    #[test]
    fn encode_u8_255_twice() {
        let mut res = String::new();
        res.write_bytes(&[255u8, 255u8]);
        assert!(&res == "//P");
    }

    #[test]
    fn encode_u8_255_thrice() {
        let mut res = String::new();
        res.write_bytes(&[255u8, 255u8, 255u8]);
        assert!(&res == "////");
    }

    #[test]
    fn encode_more_u8s() {
        let mut res = String::new();
        res.write_bytes(&[255u8, 255u8, 255u8, 0u8]);
        assert!(&res == "////AA");
    }

    #[test]
    fn encode_i32_0() {
        let mut res = String::new();
        res.write_i32(0);
        assert!(&res == "AAAAAA");
    }

    #[test]
    fn encode_i32_1() {
        let mut res = String::new();
        res.write_i32(1);
        assert!(&res == "BAAAAA");
    }

    #[test]
    fn encode_u32() {
        let mut res = String::new();
        res.write_u32(0xf00d1234);
        assert!(&res == "0IRDwD");
    }

    #[test]
    fn encode_f64() {
        let mut res = String::new();
        res.write_f64(f64::consts::PI);
        assert!(&res == "Y0CRUtfIJAE");
    }

    #[test]
    fn encode_relative() {
        let mut res = String::new();
        res.write_next_f64(f64::consts::PI, f64::consts::PI);
        assert!(&res == "AAA");
    }

    #[test]
    fn encode_small_usize() {
        let mut res = String::new();
        res.write_usize(5);
        assert!(&res == "F");
    }

    #[test]
    fn encode_small_u64() {
        let mut res = String::new();
        res.write_small_u64(5);
        assert!(&res == "F");
    }

    #[test]
    fn encode_larger_usize() {
        let mut res = String::new();
        res.write_usize(32768);
        assert!(&res == "gggB");
    }

    #[test]
    fn encode_short_duration() {
        let mut res = String::new();
        res.write_duration(Duration::from_millis(2000));
        assert!(&res == "tASoHAA");
    }

    #[test]
    fn encode_long_duration() {
        let mut res = String::new();
        res.write_duration(Duration::from_secs(8000));
        assert!(&res == "TAAl1cHAAAAA");
    }

    #[test]
    fn encode_string() {
        let mut res = String::new();
        res.write_str("Test");
        assert!(&res == "EUV2c0B");
    }
}
