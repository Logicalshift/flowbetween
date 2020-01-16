use super::target::*;
use super::super::traits::*;

impl PathComponent {
    ///
    /// Generates a serialized version of this path component on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PathComponent::*;

        match self {
            Move(p)             => { data.write_chr('M'); data.write_f64(p.position.0); data.write_f64(p.position.1); }
            Line(p)             => { data.write_chr('L'); data.write_f64(p.position.0); data.write_f64(p.position.1); }
            Bezier(p1, p2, p3)  => { data.write_chr('C'); data.write_f64(p1.position.0); data.write_f64(p1.position.1); data.write_f64(p2.position.0); data.write_f64(p2.position.1); data.write_f64(p3.position.0); data.write_f64(p3.position.1); }
            Close               => { data.write_chr('X'); }
        }
    }
}
