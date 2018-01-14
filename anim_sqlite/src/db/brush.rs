use super::*;

impl AnimationDbCore {
    ///
    /// Inserts a brush definition
    /// 
    pub fn insert_brush(sqlite: &Connection, brush_definition: &BrushDefinition, edit_log_enum: &EditLogEnumValues) -> Result<i64> {
        use animation::BrushDefinition::*;

        // As with other things that might be called a lot, rusqlites lifetime requirements on prepared statements may cause perf issues here
        let mut insert_brush_type   = sqlite.prepare_cached("INSERT INTO Flo_Brush_Type (BrushType) VALUES (?)").unwrap();
        let mut insert_brush_ink    = sqlite.prepare_cached("INSERT INTO Flo_Brush_Ink (Brush, MinWidth, MaxWidth, ScaleUpDistance) VALUES (?, ?, ?, ?)").unwrap();

        // Base brush
        let brush_type = match brush_definition {
            &Simple     => edit_log_enum.brush_simple,
            &Ink(_)     => edit_log_enum.brush_ink
        };

        let brush_id = insert_brush_type.insert(&[&brush_type])?;

        // Type-specific information
        match brush_definition {
            &Simple             => { },
            &Ink(ref ink_defn)  => { 
                insert_brush_ink.insert(&[
                    &brush_id,

                    &(ink_defn.min_width as f64),
                    &(ink_defn.max_width as f64),
                    &(ink_defn.scale_up_distance as f64)
                ])?;
            }
        }

        Ok(brush_id)
    }
}