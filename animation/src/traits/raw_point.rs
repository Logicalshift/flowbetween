use flo_float_encoder::*;
use std::io::{Read, Write, Error, ErrorKind};

///
/// A raw point represents a point from an input device
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct RawPoint {
    /// Where the pointer was on the canvas
    pub position: (f32, f32),

    /// The pressure used
    pub pressure: f32,

    /// The tilt of the device
    pub tilt: (f32, f32)
}

impl From<(f32, f32)> for RawPoint {
    fn from(pos: (f32, f32)) -> RawPoint {
        RawPoint {
            position:   pos,
            tilt:       (0.0, 0.0),
            pressure:   1.0
        }
    }
}

///
/// Writes a set of raw points to a stream
///
pub fn write_raw_points<Target: Write>(tgt: &mut Target, points: &[RawPoint]) -> Result<(), Error> {
    let mut last_position   = (0.0, 0.0);
    let mut last_pressure   = 0.0;
    let mut last_tilt       = (0.0, 0.0);

    for point in points {
        // Squish the points into the stream (we rely on the way that points are usually close together to reduce the amount of data we need to write to the stream)
        squish_float(tgt, last_position.0 as f64, point.position.0 as f64)?;
        squish_float(tgt, last_position.1 as f64, point.position.1 as f64)?;
        squish_float(tgt, last_pressure as f64, point.pressure as f64)?;
        squish_float(tgt, last_tilt.0 as f64, point.tilt.0 as f64)?;
        squish_float(tgt, last_tilt.1 as f64, point.tilt.1 as f64)?;

        // Store the previous positions (for squishing)
        last_position   = point.position;
        last_pressure   = point.pressure;
        last_tilt       = point.tilt;
    }

    Ok(())
}

///
/// Reads a set of raw points froma stream
///
pub fn read_raw_points<Source: Read>(src: &mut Source) -> Result<Vec<RawPoint>, Error> {
    let mut last_position   = (0.0, 0.0);
    let mut last_pressure   = 0.0;
    let mut last_tilt       = (0.0, 0.0);

    let mut result          = vec![];

    loop {
        // Stop if we reach the EOF while reading the X position
        let x_pos = unsquish_float(src, last_position.0 as f64);
        if x_pos.as_ref().err().map(|err| err.kind()) == Some(ErrorKind::UnexpectedEof) {
            break;
        }

        // Read the rest of the point if we have an x-pos
        let point = RawPoint {
            position:   (x_pos? as f32, unsquish_float(src, last_position.1 as f64)? as f32),
            pressure:   unsquish_float(src, last_pressure as f64)? as f32,
            tilt:       (unsquish_float(src, last_tilt.0 as f64)? as f32, unsquish_float(src, last_tilt.1 as f64)? as f32)
        };

        result.push(point);

        last_position   = point.position;
        last_pressure   = point.pressure;
        last_tilt       = point.tilt;
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_decode_raw_points() {
        let mut tgt = vec![];

        write_raw_points(&mut tgt, &vec![
            RawPoint { position: (2.0, 2.0), pressure: 0.5, tilt: (0.0, 0.0) },
            RawPoint { position: (2.0, 2.4), pressure: 0.55, tilt: (20.0, 0.0) },
            RawPoint { position: (2.7, 4.2), pressure: 0.51, tilt: (20.0, 11.0) },
            RawPoint { position: (3.0, 2.0), pressure: 0.3, tilt: (3.0, 4.0) },
        ]).unwrap();

        assert!(tgt.len() == 40);

        let mut src: &[u8] = &tgt;
        let read_points = read_raw_points(&mut src).unwrap();
        assert!(read_points.len() == 4);

        assert!((read_points[1].position.1-2.4).abs() < 0.01);
        assert!((read_points[2].position.0-2.7).abs() < 0.01);
        assert!((read_points[2].tilt.0-20.0).abs() < 0.01);
        assert!((read_points[2].tilt.1-11.0).abs() < 0.01);
        assert!((read_points[3].pressure-0.3).abs() < 0.01);
    }
}
