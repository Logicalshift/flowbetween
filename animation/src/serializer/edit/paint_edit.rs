use super::super::target::*;
use super::super::super::traits::*;

impl PaintEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PaintEdit::*;

        match self {
            SelectBrush(elem, brush_defn, drawing_style)    => { data.write_chr('*'); elem.serialize(data); unimplemented!("SelectBrush") }
            BrushProperties(elem, props)                    => { data.write_chr('P'); elem.serialize(data); unimplemented!("BrushProperties") }
            BrushStroke(elem, points)                       => { data.write_chr('S'); elem.serialize(data); unimplemented!("BrushStroke") }
        }
    }
}
