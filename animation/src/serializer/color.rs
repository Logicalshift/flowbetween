use super::target::*;

use flo_canvas::*;

///
/// Generates a serialized version of a colour on the specified data target
///
pub fn serialize_color<Tgt: AnimationDataTarget>(color: &Color, data: &mut Tgt) {
    use self::Color::*;

    match color {
        Rgba(r, g, b, a)    => { data.write_chr('R'); data.write_f32(*r); data.write_f32(*g); data.write_f32(*b); data.write_f32(*a); }
        Hsluv(h, s, l, a)   => { data.write_chr('H'); data.write_f32(*h); data.write_f32(*s); data.write_f32(*l); data.write_f32(*a); }
    }
}
