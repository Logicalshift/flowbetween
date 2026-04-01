use std::io::{Read, Error};
use std::mem;

///
/// Reads a squished float from a source stream
///
pub fn unsquish_float<Source: Read>(src: &mut Source, last: f64) -> Result<f64, Error> {
    let last = if last.is_infinite() || last.is_nan() { 0.0 } else { last };

    let nan: u16        = 0b1000_0000_0000_0000;

    // Read the fixed-point diff
    let mut diff_bytes  = [0,0];
    src.read_exact(&mut diff_bytes)?;
    let diff: i16       = (diff_bytes[0] as i16) | ((diff_bytes[1] as i16)<<8);

    if diff == (nan as i16) {
        // Diff stored as a f32
        let mut diff_bytes          = [0,0,0,0];
        src.read_exact(&mut diff_bytes)?;
        let diff_bit_pattern: u32   =
              ((diff_bytes[0] as u32)<<0)
            | ((diff_bytes[1] as u32)<<8)
            | ((diff_bytes[2] as u32)<<16)
            | ((diff_bytes[3] as u32)<<24);

        let diff: f32 = unsafe { mem::transmute(diff_bit_pattern) };

        Ok(last + (diff as f64))
    } else {
        // Turn the diff into a float
        let diff = (diff as f64) / 256.0;
        Ok(last + diff)
    }
}
