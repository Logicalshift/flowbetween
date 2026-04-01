use std::io::{Write, Error};
use std::mem;

/// Maximum distance to use for f16 difference encoding (larger differences increase data squishing but reduce precision)
const MAX_DISTANCE: f32 = 126.0;

///
/// Writes a squished float to the specified stream
///
/// Floats are stored relative to their previous value. Any 'initial' value can
/// be chosen (provided it's re-used when de-squishing). 0.0 will work well.
///
pub fn squish_float<Target: Write>(target: &mut Target, last: f64, next: f64) -> Result<usize, Error> {
    let last = if last.is_infinite() || last.is_nan() { 0.0 } else { last };

    // What we encode is the difference between two floats
    let diff = (next - last) as f32;

    if last.is_nan() || next.is_nan() || diff.abs() > MAX_DISTANCE {
        // For 'bad' floats we encode as f16 NAN, f32 val
        let nan: u16            = 0b1000_0000_0000_0000;
        let bit_pattern: u32    = unsafe { mem::transmute(diff) };

        target.write(&[(nan&0xff) as u8, (nan>>8) as u8])?;
        target.write(&[
            ((bit_pattern>>0) &0xff) as u8,
            ((bit_pattern>>8) &0xff) as u8,
            ((bit_pattern>>16)&0xff) as u8,
            ((bit_pattern>>24)&0xff) as u8
        ])
    } else {
        // For 'good' floats we encode as a 16-bit fixed point number
        let fixed_point = (diff * 256.0) as i16;

        target.write(&[(fixed_point&0xff) as u8, (fixed_point>>8) as u8])
    }
}
