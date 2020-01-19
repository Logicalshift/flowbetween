use smallvec::*;

use std::str::{Chars};
use std::convert::{TryInto};
use std::time::{Duration};

///
/// Decodes a character to a 6-bit value (ie, from ENCODING_CHAR_SET)
///
fn decode_chr(c: char) -> u8 {
    if c >= 'A' && c <= 'Z' {
        ((c as u8) - ('A' as u8)) as u8
    } else if c >= 'a' && c <= 'z' {
        (((c as u8) - ('a' as u8)) as u8) + 26
    } else if c >= '0' && c <= '9' {
        (((c as u8) - ('0' as u8)) as u8) + 52
    } else if c == '+' {
        62
    } else if c == '/' {
        63
    } else {
        0
    }
}

///
/// Represents a source for serialized animation data
///
pub trait AnimationDataSource {
    ///
    /// Reads the next character from this data source
    ///
    fn next_chr(&mut self) -> char;

    ///
    /// Reads len bytes from this data source
    ///
    fn next_bytes(&mut self, len: usize) -> SmallVec<[u8;8]> {
        // Build the result into a smallvec
        let mut res     = smallvec![];

        // Current value and bit pos
        let mut current = 0u8;
        let mut bit_pos = 0;

        // Iterate until we've read enough bytes from the source
        while res.len() < len {
            // Process a character from the input
            let byte    = decode_chr(self.next_chr());

            if bit_pos + 6 >= 8 {
                // Add the remaining bits the current value
                let mask    = (1<<(8-bit_pos))-1;
                current     |= (byte&mask)<<bit_pos;

                res.push(current);

                // Remaining bits go in 'current'
                current = byte >> (8-bit_pos);
                bit_pos = 6-(8-bit_pos);
            } else {
                // Just add the bits to 'current' and carry on
                current |= byte << bit_pos;
                bit_pos += 6;
            }
        }

        res
    }

    ///
    /// Reads a usize from this source
    ///
    fn next_usize(&mut self) -> usize {
        let mut result  = 0usize;
        let mut shift   = 0;

        loop {
            // Decode the next character
            let byte    = decode_chr(self.next_chr());

            // Lower 5 bits contain the value
            let lower   = byte & 0x1f;
            result      |= (lower as usize) << shift;
            shift       += 5;

            // If the upper bit is set there are more bytes
            if (byte & 0x20) == 0 {
                break;
            }
        }

        result
    }

    ///
    /// Reads a u64 (in the format where it's expected to have a small value) from this source
    ///
    fn next_small_u64(&mut self) -> u64 {
        let mut result  = 0u64;
        let mut shift   = 0;

        loop {
            // Decode the next character
            let byte    = decode_chr(self.next_chr());

            // Lower 5 bits contain the value
            let lower   = byte & 0x1f;
            result      |= (lower as u64) << shift;
            shift       += 5;

            // If the upper bit is set there are more bytes
            if (byte & 0x20) == 0 {
                break;
            }
        }

        result
    }

    ///
    /// Reads a string from the source
    ///
    fn next_string(&mut self) -> String {
        let num_bytes   = self.next_usize();
        let utf8        = self.next_bytes(num_bytes);

        String::from_utf8_lossy(&utf8).to_string()
    }

    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(*&self.next_bytes(4)[0..4].try_into().unwrap())
    }

    fn next_i32(&mut self) -> i32 {
        i32::from_le_bytes(*&self.next_bytes(4)[0..4].try_into().unwrap())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(*&self.next_bytes(8)[0..8].try_into().unwrap())
    }

    fn next_i64(&mut self) -> i64 {
        i64::from_le_bytes(*&self.next_bytes(8)[0..8].try_into().unwrap())
    }

    fn next_f32(&mut self) -> f32 {
        f32::from_le_bytes(*&self.next_bytes(4)[0..4].try_into().unwrap())
    }

    fn next_f64(&mut self) -> f64 {
        f64::from_le_bytes(*&self.next_bytes(8)[0..8].try_into().unwrap())
    }

    ///
    /// Writes a time to the target
    ///
    fn next_duration(&mut self) -> Duration {
        match self.next_chr() {
            't'     => Duration::from_micros(self.next_u32() as u64),
            'T'     => Duration::from_micros(self.next_u64()),
            _       => Duration::from_millis(0)
        }
    }
}

impl AnimationDataSource for Chars<'_> {
    fn next_chr(&mut self) -> char {
        self.next().unwrap_or('A')
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::target::*;

    #[test]
    fn decode_bytes() {
        let bytes       = (0u8..=255u8).into_iter().collect::<Vec<_>>();
        let mut encoded = String::new();

        encoded.write_bytes(&bytes);

        for end in 0u8..255u8 {
            let decoded     = encoded.chars().next_bytes((end as usize)+1);
            assert!(decoded == (0u8..=end).into_iter().collect::<SmallVec<[u8; 8]>>());
        }
    }

    #[test]
    fn decode_usize() {
        let mut encoded = String::new();

        encoded.write_usize(1234);
        assert!(encoded.chars().next_usize() == 1234);
    }

    #[test]
    fn decode_small_u64() {
        let mut encoded = String::new();

        encoded.write_small_u64(1234);
        assert!(encoded.chars().next_small_u64() == 1234);
    }

    #[test]
    fn decode_string() {
        let mut encoded = String::new();

        encoded.write_str("Hello, world");
        assert!(encoded.chars().next_string() == "Hello, world".to_string());
    }

    #[test]
    fn decode_u32() {
        let mut encoded = String::new();

        encoded.write_u32(1234);
        assert!(encoded.chars().next_u32() == 1234);
    }

    #[test]
    fn decode_i32() {
        let mut encoded = String::new();

        encoded.write_i32(-1234);
        assert!(encoded.chars().next_i32() == -1234);
    }

    #[test]
    fn decode_u64() {
        let mut encoded = String::new();

        encoded.write_u64(1234);
        assert!(encoded.chars().next_u64() == 1234);
    }

    #[test]
    fn decode_i64() {
        let mut encoded = String::new();

        encoded.write_i64(-1234);
        assert!(encoded.chars().next_i64() == -1234);
    }

    #[test]
    fn decode_f32() {
        let mut encoded = String::new();

        encoded.write_f32(1234.1234);
        assert!(encoded.chars().next_f32() == 1234.1234);
    }

    #[test]
    fn decode_f64() {
        let mut encoded = String::new();

        encoded.write_f64(1234.1234);
        assert!(encoded.chars().next_f64() == 1234.1234);
    }

    #[test]
    fn decode_short_duration() {
        let mut encoded = String::new();

        encoded.write_duration(Duration::from_millis(1234));
        assert!(encoded.chars().next_duration() == Duration::from_millis(1234));
    }

    #[test]
    fn decode_long_duration() {
        let mut encoded = String::new();

        encoded.write_duration(Duration::from_secs(1000000));
        assert!(encoded.chars().next_duration() == Duration::from_secs(1000000));
    }
}