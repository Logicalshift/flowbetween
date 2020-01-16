use super::target::*;

use flo_canvas::*;

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
