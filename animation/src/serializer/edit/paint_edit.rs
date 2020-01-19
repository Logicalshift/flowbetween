use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

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

    ///
    /// Deserializes a paint edit from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<PaintEdit> {
        match data.next_chr() {
            '*' => { 
                ElementId::deserialize(data).and_then(|elem_id| 
                    BrushDefinition::deserialize(data).and_then(move |brush_defn|
                        BrushDrawingStyle::deserialize(data).map(move |drawing_style|
                            PaintEdit::SelectBrush(elem_id, brush_defn, drawing_style)))) 
                }
            'P' => {
                ElementId::deserialize(data).and_then(|elem_id|
                    BrushProperties::deserialize(data).map(move |props| 
                        PaintEdit::BrushProperties(elem_id, props)))
            }

            'S' => {
                ElementId::deserialize(data).and_then(|elem_id|
                    match data.next_small_u64() {
                        0 => { 
                            // v0
                            let num_points      = data.next_usize();
                            let mut last_pos    = RawPoint::from((0.0, 0.0));
                            let mut points      = Vec::with_capacity(num_points);

                            for _point_num in 0..num_points {
                                let position    = (data.next_f64_offset(last_pos.position.0 as f64), data.next_f64_offset(last_pos.position.1 as f64));
                                let pressure    = data.next_f64_offset(last_pos.pressure as f64);
                                let tilt        = (data.next_f64_offset(last_pos.tilt.0 as f64), data.next_f64_offset(last_pos.tilt.1 as f64));

                                let next_point  = RawPoint { position: (position.0 as f32, position.1 as f32), pressure: pressure as f32, tilt: (tilt.0 as f32, tilt.1 as f32) };
                                points.push(next_point);

                                last_pos        = next_point;
                            }

                            Some(PaintEdit::BrushStroke(elem_id, Arc::new(points)))
                        }

                        _ => None
                    })
            }

            _   => None
        }
    }
}

