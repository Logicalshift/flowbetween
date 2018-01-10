use super::*;
use super::editlog_statements::*;

use animation::*;

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
}

impl AnimationDb {

}

impl AnimationDbCore {

}

