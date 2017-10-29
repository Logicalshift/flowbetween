///
/// Trait implemented by objects that can be encoded into a canvas
///
pub trait CanvasEncoding {
    ///
    /// Encodes this item by appending it to the specified string
    ///
    fn encode_canvas(&self, append_to: &mut String);
}

const ENCODING_CHAR_SET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'
];

impl CanvasEncoding for u32 {
    fn encode_canvas(&self, append_to: &mut String) {
        // Base-64 wastes some bits but requires 2 less characters than hex for a 32-bit number
        let mut remaining = *self;

        for _ in 0..6 {
            let next_part = remaining & 0x3f;
            let next_char = ENCODING_CHAR_SET[next_part as usize];
            append_to.push(next_char);

            remaining >>= 6;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_encode_u32() {
        let test_number: u32 = 0xabcd1234;

        let mut encoded = String::new();
        test_number.encode_canvas(&mut encoded);

        assert!(encoded == "0IRzrC".to_string());
    }
}
