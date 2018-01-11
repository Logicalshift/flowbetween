use super::*;
use super::editlog_statements::*;

use canvas::*;
use animation::*;

use std::time::Duration;

///
/// The values to use for the enum values in the various editlog tables
/// 
pub struct EditLogEnumValues {
    pub set_size:                       i32,
    pub add_new_layer:                  i32,
    pub remove_layer:                   i32,
    pub layer_add_keyframe:             i32,
    pub layer_remove_keyframe:          i32,
    pub layer_paint_select_brush:       i32,
    pub layer_paint_brush_properties:   i32,
    pub layer_paint_brush_stroke:       i32,
 
    pub draw_draw:                      i32,
    pub draw_erase:                     i32,
 
    pub brush_simple:                   i32,
    pub brush_ink:                      i32,
 
    pub color_rgb:                      i32,
    pub color_hsluv:                    i32
}

impl EditLogEnumValues {
    ///
    /// Cretes a new edit log values structure by discovering the enum values from the specified connection
    /// 
    pub fn new(sqlite: &Connection) -> EditLogEnumValues {
        let mut read_field = sqlite.prepare("SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = ? AND ApiName = ?").unwrap();

        let mut value_for_enum = |field_name: &str, enum_name: &str| {
            read_field.query_row(&[&field_name, &enum_name], |row| row.get(0)).unwrap()
        };

        EditLogEnumValues {
            set_size:                       value_for_enum("Edit", "SetSize"),
            add_new_layer:                  value_for_enum("Edit", "AddNewLayer"),
            remove_layer:                   value_for_enum("Edit", "RemoveLayer"),
            layer_add_keyframe:             value_for_enum("Edit", "Layer::AddKeyFrame"),
            layer_remove_keyframe:          value_for_enum("Edit", "Layer::RemoveKeyFrame"),
            layer_paint_select_brush:       value_for_enum("Edit", "Layer::Paint::SelectBrush"),
            layer_paint_brush_properties:   value_for_enum("Edit", "Layer::Paint::BrushProperties"),
            layer_paint_brush_stroke:       value_for_enum("Edit", "Layer::Paint::BrushStroke"),

            draw_draw:                      value_for_enum("DrawingStyle", "Draw"),
            draw_erase:                     value_for_enum("DrawingStyle", "Erase"),

            brush_simple:                   value_for_enum("BrushType", "Simple"),
            brush_ink:                      value_for_enum("BrushType", "Ink"),

            color_rgb:                      value_for_enum("ColorType", "RGB"),
            color_hsluv:                    value_for_enum("ColorType", "HSLUV")
        }
    }

    ///
    /// Retrieves the edit type for an animation edit
    /// 
    pub fn edit_type(&self, edit: &AnimationEdit) -> i32 {
        use animation::AnimationEdit::*;
        use animation::LayerEdit::*;
        use animation::PaintEdit::*;

        match edit {
            &SetSize(_, _)                              => self.set_size,
            &AddNewLayer(_)                             => self.add_new_layer,
            &RemoveLayer(_)                             => self.remove_layer,

            &Layer(_, AddKeyFrame(_))                   => self.layer_add_keyframe,
            &Layer(_, RemoveKeyFrame(_))                => self.layer_remove_keyframe,

            &Layer(_, Paint(_, SelectBrush(_, _)))      => self.layer_paint_select_brush,
            &Layer(_, Paint(_, BrushProperties(_)))     => self.layer_paint_brush_properties,
            &Layer(_, Paint(_, BrushStroke(_)))         => self.layer_paint_brush_stroke
        }
    }
}

impl AnimationDb {

}

impl AnimationDbCore {
    ///
    /// Inserts a set of edits into the database
    /// 
    pub fn insert_edits(&mut self, edits: &[AnimationEdit]) -> Result<()> {
        // Make sure the enum values are available
        let sqlite = &self.sqlite;
        self.edit_log_enum.get_or_insert_with(|| {
            EditLogEnumValues::new(sqlite)
        });

        // Statement cache
        let mut statements = EditLogStatements::new(sqlite);

        // Insert all of the edits in turn
        for edit in edits {
            self.insert_edit_log(&mut statements, edit)?;
        }

        Ok(())
    }

    ///
    /// Inserts a single AnimationEdit into the edit log
    /// 
    fn insert_edit_log<'a>(&self, statements: &mut EditLogStatements<'a>, edit: &AnimationEdit) -> Result<i64> {
        // Create the basic edit
        let type_id = self.edit_log_enum.as_ref().unwrap().edit_type(edit);
        let edit_id = statements.insert_editlog().insert(&[&type_id])?;

        // Insert the values for this edit
        self.insert_animation_edit(statements, edit_id, edit)?;

        Ok(edit_id)
    }

    ///
    /// Inserts the values for an AnimationEdit into the edit log
    /// 
    fn insert_animation_edit<'a>(&self, statements: &mut EditLogStatements<'a>, edit_id: i64, edit: &AnimationEdit) -> Result<()> {
        use animation::AnimationEdit::*;

        match edit {
            &Layer(layer_id, ref layer_edit)    => { 
                statements.insert_el_layer().insert(&[&edit_id, &(layer_id as i64)])?;
                self.insert_layer_edit(statements, edit_id, layer_edit)?;
            },

            &SetSize(width, height)             => { 
                statements.insert_el_size().insert(&[&edit_id, &width, &height])?;
            },

            &AddNewLayer(layer_id)              => { 
                statements.insert_el_layer().insert(&[&edit_id, &(layer_id as i64)])?;
            },

            &RemoveLayer(layer_id)              => {
                statements.insert_el_layer().insert(&[&edit_id, &(layer_id as i64)])?;
            }
        };

        Ok(())
    }

    ///
    /// Retrieves microseconds from a duration
    /// 
    fn get_micros(when: &Duration) -> i64 {
        let secs:i64    = when.as_secs() as i64;
        let nanos:i64   = when.subsec_nanos() as i64;

        (secs * 1_000_000) + (nanos / 1_000)
    }

    ///
    /// Inserts the values for a LayerEdit into the edit log
    /// 
    fn insert_layer_edit<'a>(&self, statements: &mut EditLogStatements<'a>, edit_id: i64, edit: &LayerEdit) -> Result<()> {
        use animation::LayerEdit::*;

        match edit {
            &Paint(when, ref paint_edit)    => {
                statements.insert_el_when().insert(&[&edit_id, &Self::get_micros(&when)])?;
                self.insert_paint_edit(statements, edit_id, paint_edit)?;
            }

            &AddKeyFrame(when)              => {
                statements.insert_el_when().insert(&[&edit_id, &Self::get_micros(&when)])?;
            }

            &RemoveKeyFrame(when)           => {
                statements.insert_el_when().insert(&[&edit_id, &Self::get_micros(&when)])?;
            }
        }

        Ok(())
    }

    ///
    /// Inserts the values for a LayerEdit into the edit log
    /// 
    fn insert_paint_edit<'a>(&self, statements: &mut EditLogStatements<'a>, edit_id: i64, edit: &PaintEdit) -> Result<()> {
        use animation::PaintEdit::*;

        match edit {
            &SelectBrush(ref definition, ref drawing_style) => {
                let brush_id        = self.insert_brush(statements, definition)?;
                let drawing_style   = match drawing_style {
                    &BrushDrawingStyle::Draw    => self.edit_log_enum.as_ref().unwrap().draw_draw,
                    &BrushDrawingStyle::Erase   => self.edit_log_enum.as_ref().unwrap().draw_erase
                };
                statements.insert_el_brush().insert(&[&edit_id, &drawing_style, &brush_id])?;
            },

            &BrushProperties(ref properties)                => {
                let color_id        = self.insert_color(statements, &properties.color)?;
                statements.insert_brush_properties().insert(&[
                    &edit_id, 
                    
                    &(properties.size as f64),
                    &(properties.opacity as f64),
                    &color_id
                ])?;
            },

            &BrushStroke(ref points)                        => {
                self.insert_raw_points(statements, edit_id, &**points)?;
            }
        }

        Ok(())
    }

    ///
    /// Inserts a set of raw points into this itme
    /// 
    fn insert_raw_points<'a>(&self, statements: &mut EditLogStatements<'a>, edit_id: i64, points: &[RawPoint]) -> Result<()> {
        let insert_point = statements.insert_el_rawpoint();

        for (point, index) in points.iter().zip(0..(points.len() as i64)) {
            insert_point.insert(&[
                &edit_id,
                &index,

                &(point.position.0 as f64), 
                &(point.position.1 as f64), 
                &(point.pressure as f64), 
                &(point.tilt.0 as f64), 
                &(point.tilt.1 as f64)
            ])?;
        }

        Ok(())
    }

    ///
    /// Inserts a brush definition
    /// 
    pub fn insert_brush<'a>(&self, statements: &mut EditLogStatements<'a>, brush_definition: &BrushDefinition) -> Result<i64> {
        use animation::BrushDefinition::*;

        // Base brush
        let brush_type = match brush_definition {
            &Simple     => self.edit_log_enum.as_ref().unwrap().brush_simple,
            &Ink(_)     => self.edit_log_enum.as_ref().unwrap().brush_ink
        };

        let brush_id = statements.insert_brush_type().insert(&[&brush_type])?;

        // Type-specific information
        match brush_definition {
            &Simple             => { },
            &Ink(ref ink_defn)  => { 
                statements.insert_brush_ink().insert(&[
                    &brush_id,

                    &(ink_defn.min_width as f64),
                    &(ink_defn.max_width as f64),
                    &(ink_defn.scale_up_distance as f64)
                ])?;
            }
        }

        Ok(brush_id)
    }
    
    ///
    /// Inserts a colour definition
    /// 
    pub fn insert_color<'a>(&self, statements: &mut EditLogStatements<'a>, color: &Color) -> Result<i64> {
        // Base colour
        let color_type = match color {
            &Color::Rgba(_, _, _, _)    => self.edit_log_enum.as_ref().unwrap().color_rgb,
            &Color::Hsluv(_, _, _, _)   => self.edit_log_enum.as_ref().unwrap().color_hsluv,
        };

        let color_id = statements.insert_color_type().insert(&[&color_type])?;

        // Components
        match color {
            &Color::Rgba(r, g, b, _) => {
                statements.insert_color_rgb().insert(&[
                    &color_id,
                    &(r as f64),
                    &(g as f64),
                    &(b as f64)
                ])?;
            },
            &Color::Hsluv(h, s, l, _) => {
                statements.insert_color_hsluv().insert(&[
                    &color_id,
                    &(h as f64),
                    &(s as f64),
                    &(l as f64)
                ])?;
            },
        }

        Ok(color_id)
    }
}
