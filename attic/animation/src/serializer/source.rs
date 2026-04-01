use smallvec::*;
use flo_float_encoder::*;

use std::io;
use std::io::{Read};
use std::str::{Chars};
use std::time::{Duration};
use std::convert::{TryInto};

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
/// Reader implementation that can read bytes from an animation data source
///
struct ByteReader<'a, Src: AnimationDataSource> {
    /// The data source (this will call next_chr only so this is suitable for implementing next_bytes)
    src: &'a mut Src,

    /// The current byte
    current: u8,

    /// The current bit pos
    bit_pos: usize
}

impl<'a, Src: AnimationDataSource> Read for ByteReader<'a, Src> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let mut pos = 0;

        // Iterate until we've read enough bytes from the source
        while pos < buf.len() {
            // Process a character from the input
            let byte    = decode_chr(self.src.next_chr());

            if self.bit_pos + 6 >= 8 {
                // Add the remaining bits the current value
                let mask        = (1<<(8-self.bit_pos))-1;
                self.current    |= (byte&mask)<<self.bit_pos;

                buf[pos] = self.current;
                pos += 1;

                // Remaining bits go in 'current'
                self.current    = byte >> (8-self.bit_pos);
                self.bit_pos    = 6-(8-self.bit_pos);
            } else {
                // Just add the bits to 'current' and carry on
                self.current    |= byte << self.bit_pos;
                self.bit_pos    += 6;
            }
        }

        Ok(pos)
    }
}

///
/// Represents a source for serialized animation data
///
pub trait AnimationDataSource : Sized {
    ///
    /// Reads the next character from this data source
    ///
    fn next_chr(&mut self) -> char;

    ///
    /// Reads len bytes from this data source
    ///
    fn next_bytes(&mut self, len: usize) -> SmallVec<[u8;8]> {
        // Build the result into a smallvec
        let mut res     = smallvec![0; len];

        // Read using a ByteReader (defined above)
        let mut reader  = ByteReader { src: self, current: 0, bit_pos: 0 };
        reader.read(&mut res[0..len]).ok();

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
    /// Writes a f64 value to this target that's relative to a previous value (this uses a more compact format to save space)
    ///
    fn next_f64_offset(&mut self, last: f64) -> f64 {
        // Read using a ByteReader (defined above)
        let mut reader  = ByteReader { src: self, current: 0, bit_pos: 0 };

        unsquish_float(&mut reader, last).unwrap()
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

    #[test]
    fn decode_f64_small_offset() {
        let mut encoded = String::new();

        encoded.write_next_f64(14.0, 64.0);
        assert!(encoded.chars().next_f64_offset(14.0) == 64.0);
    }

    #[test]
    fn decode_f64_large_offset() {
        let mut encoded = String::new();

        encoded.write_next_f64(14.0, 64000.0);
        assert!(encoded.chars().next_f64_offset(14.0) == 64000.0);
    }

    #[test]
    fn decode_all() {
        // Encodes everything next to each other so we know the decoder is always left in a valid state afterwards (doesn't read an extra character)
        let mut encoded = String::new();

        encoded.write_usize(1234);
        encoded.write_small_u64(1234);
        encoded.write_str("Hello, world");
        encoded.write_u32(1234);
        encoded.write_i32(-1234);
        encoded.write_u64(1234);
        encoded.write_i64(-1234);
        encoded.write_f32(1234.1234);
        encoded.write_f64(1234.1234);
        encoded.write_duration(Duration::from_millis(1234));
        encoded.write_duration(Duration::from_secs(1000000));
        encoded.write_next_f64(14.0, 64.0);
        encoded.write_next_f64(14.0, 64000.0);

        encoded.write_usize(1234);
        encoded.write_small_u64(1234);
        encoded.write_str("Hello, world");
        encoded.write_u32(1234);
        encoded.write_i32(-1234);
        encoded.write_u64(1234);
        encoded.write_i64(-1234);
        encoded.write_f32(1234.1234);
        encoded.write_f64(1234.1234);
        encoded.write_duration(Duration::from_millis(1234));
        encoded.write_duration(Duration::from_secs(1000000));
        encoded.write_next_f64(14.0, 64.0);
        encoded.write_next_f64(14.0, 64000.0);

        let mut src = encoded.chars();

        assert!(src.next_usize() == 1234);
        assert!(src.next_small_u64() == 1234);
        assert!(src.next_string() == "Hello, world".to_string());
        assert!(src.next_u32() == 1234);
        assert!(src.next_i32() == -1234);
        assert!(src.next_u64() == 1234);
        assert!(src.next_i64() == -1234);
        assert!(src.next_f32() == 1234.1234);
        assert!(src.next_f64() == 1234.1234);
        assert!(src.next_duration() == Duration::from_millis(1234));
        assert!(src.next_duration() == Duration::from_secs(1000000));
        assert!(src.next_f64_offset(14.0) == 64.0);
        assert!(src.next_f64_offset(14.0) == 64000.0);

        assert!(src.next_usize() == 1234);
        assert!(src.next_small_u64() == 1234);
        assert!(src.next_string() == "Hello, world".to_string());
        assert!(src.next_u32() == 1234);
        assert!(src.next_i32() == -1234);
        assert!(src.next_u64() == 1234);
        assert!(src.next_i64() == -1234);
        assert!(src.next_f32() == 1234.1234);
        assert!(src.next_f64() == 1234.1234);
        assert!(src.next_duration() == Duration::from_millis(1234));
        assert!(src.next_duration() == Duration::from_secs(1000000));
        assert!(src.next_f64_offset(14.0) == 64.0);
        assert!(src.next_f64_offset(14.0) == 64000.0);
    }
}
