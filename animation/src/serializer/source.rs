use smallvec::*;

use std::str::{Chars};

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
}
