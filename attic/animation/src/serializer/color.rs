use super::source::*;
use super::target::*;

use flo_canvas::*;

use smallvec::*;
use std::convert::*;

///
/// Generates a serialized version of a colour on the specified data target
///
pub fn serialize_color<Tgt: AnimationDataTarget>(color: &Color, data: &mut Tgt) {
    use self::Color::*;

    // a, b, c, d are float values >= 0.0 and <= 1.0: this generates a compact byte representation
    fn small_quad((a, b, c, d): (f32, f32, f32, f32)) -> Vec<u8> {
        let mut result = vec![];

        result.extend(((a*65535.0) as u16).to_le_bytes().iter());
        result.extend(((b*65535.0) as u16).to_le_bytes().iter());
        result.extend(((c*65535.0) as u16).to_le_bytes().iter());
        result.extend(((d*65535.0) as u16).to_le_bytes().iter());

        result
    }

    match color {
        Rgba(r, g, b, a)    => { 
            if r >= &0.0 && r <= &1.0 && g >= &0.0 && g <= &1.0 && b >= &0.0 && b <= &1.0 && a >= &0.0 && a <= &1.0 {
                data.write_chr('r');
                data.write_bytes(&small_quad((*r, *g, *b, *a)));
            } else {
                data.write_chr('R'); 
                data.write_f32(*r); data.write_f32(*g); data.write_f32(*b); data.write_f32(*a);
            }
        }

        Hsluv(h, s, l, a)   => { 
            if h >= &0.0 && h <= &1.0 && s >= &0.0 && s <= &1.0 && l >= &0.0 && l <= &1.0 && a >= &0.0 && a <= &1.0 {
                data.write_chr('h');
                data.write_bytes(&small_quad((*h, *s, *l, *a)));
            } else {
                data.write_chr('H'); 
                data.write_f32(*h); data.write_f32(*s); data.write_f32(*l); data.write_f32(*a);
            }
        }
    }
}

///
/// Deserializes a colour from a data source
///
pub fn deserialize_color<Src: AnimationDataSource>(data: &mut Src) -> Option<Color> {
    fn read_small_quad(bytes: SmallVec<[u8; 8]>) -> (f32, f32, f32, f32) {
        let a = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
        let b = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
        let c = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        let d = u16::from_le_bytes(bytes[6..8].try_into().unwrap());

        let a = (a as f32) / 65535.0;
        let b = (b as f32) / 65535.0;
        let c = (c as f32) / 65535.0;
        let d = (d as f32) / 65535.0;

        (a, b, c, d)
    }

    match data.next_chr() {
        'R' => Some(Color::Rgba(data.next_f32(), data.next_f32(), data.next_f32(), data.next_f32())),
        'H' => Some(Color::Hsluv(data.next_f32(), data.next_f32(), data.next_f32(), data.next_f32())),
        'r' => { let (r, g, b, a) = read_small_quad(data.next_bytes(8)); Some(Color::Rgba(r, g, b, a)) },
        'h' => { let (h, s, l, a) = read_small_quad(data.next_bytes(8)); Some(Color::Hsluv(h, s, l, a)) }
        _ => None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rgba_1() {
        let mut encoded = String::new();
        serialize_color(&Color::Rgba(0.25, 0.5, 0.75, 1.0), &mut encoded);

        if let Some(Color::Rgba(a, b, c, d)) = deserialize_color(&mut encoded.chars()) {
            assert!((a-0.25).abs() < 0.001);
            assert!((b-0.5).abs() < 0.001);
            assert!((c-0.75).abs() < 0.001);
            assert!((d-1.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn rgba_2() {
        let mut encoded = String::new();
        serialize_color(&Color::Rgba(2.25, 2.5, 2.75, 3.0), &mut encoded);

        assert!(deserialize_color(&mut encoded.chars()) == Some(Color::Rgba(2.25, 2.5, 2.75, 3.0)));
    }

    #[test]
    fn rgba_3() {
        if let Some(Color::Rgba(a, b, c, d)) = deserialize_color(&mut "r//z//9/v//P".chars()) {
            assert!((a-0.25).abs() < 0.001);
            assert!((b-0.5).abs() < 0.001);
            assert!((c-0.75).abs() < 0.001);
            assert!((d-1.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn hsluv_1() {
        let mut encoded = String::new();
        serialize_color(&Color::Hsluv(0.25, 0.5, 0.75, 1.0), &mut encoded);

        if let Some(Color::Hsluv(a, b, c, d)) = deserialize_color(&mut encoded.chars()) {
            assert!((a-0.25).abs() < 0.001);
            assert!((b-0.5).abs() < 0.001);
            assert!((c-0.75).abs() < 0.001);
            assert!((d-1.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn hsluv_2() {
        let mut encoded = String::new();
        serialize_color(&Color::Hsluv(2.25, 2.5, 2.75, 3.0), &mut encoded);

        assert!(deserialize_color(&mut encoded.chars()) == Some(Color::Hsluv(2.25, 2.5, 2.75, 3.0)));
    }

    #[test]
    fn hsluv_3() {
        if let Some(Color::Hsluv(a, b, c, d)) = deserialize_color(&mut "h//z//9/v//P".chars()) {
            assert!((a-0.25).abs() < 0.001);
            assert!((b-0.5).abs() < 0.001);
            assert!((c-0.75).abs() < 0.001);
            assert!((d-1.0).abs() < 0.001);
        } else {
            assert!(false);
        }
    }
}
