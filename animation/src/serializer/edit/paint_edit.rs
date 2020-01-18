use super::super::target::*;
use super::super::super::traits::*;

impl PaintEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PaintEdit::*;

        match self {
            SelectBrush(elem, brush_defn, drawing_style)    => { data.write_chr('*'); elem.serialize(data); brush_defn.serialize(data); drawing_style.serialize(data); }
            BrushProperties(elem, props)                    => { data.write_chr('P'); elem.serialize(data); props.serialize(data); }
            
            BrushStroke(elem, points)                       => {
                data.write_chr('S'); 
                elem.serialize(data); 

                // Version 0
                data.write_small_u64(0);

                data.write_usize(points.len());
                let mut last_pos = RawPoint::from((0.0, 0.0));

                for point in points.iter() {
                    data.write_next_f64(last_pos.position.0 as f64, point.position.0 as f64);
                    data.write_next_f64(last_pos.position.1 as f64, point.position.1 as f64);
                    data.write_next_f64(last_pos.pressure as f64, point.pressure as f64);
                    data.write_next_f64(last_pos.tilt.0 as f64, point.tilt.0 as f64);
                    data.write_next_f64(last_pos.tilt.1 as f64, point.tilt.1 as f64);

                    last_pos = *point;
                }
            }
        }
    }
}

