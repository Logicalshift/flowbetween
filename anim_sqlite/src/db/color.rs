use super::*;
use super::editlog::*;

use canvas::*;

impl AnimationDbCore {
    ///
    /// Inserts a colour definition
    /// 
    pub fn insert_color(sqlite: &Connection, color: &Color, edit_log_enum: &EditLogEnumValues) -> Result<i64> {
        // Prepared statements (rusqlite doesn't have a way we can cache them ourselves due to lifetime requirements)
        let mut insert_color_type   = sqlite.prepare_cached("INSERT INTO Flo_Color_Type (ColorType) VALUES (?)").unwrap();
        let mut insert_color_rgb    = sqlite.prepare_cached("INSERT INTO Flo_Color_Rgb (Color, R, G, B) VALUES (?, ?, ?, ?)").unwrap();
        let mut insert_color_hsluv  = sqlite.prepare_cached("INSERT INTO Flo_Color_Hsluv (Color, H, S, L) VALUES (?, ?, ?, ?)").unwrap();

        // Base colour
        let color_type = match color {
            &Color::Rgba(_, _, _, _)    => edit_log_enum.color_rgb,
            &Color::Hsluv(_, _, _, _)   => edit_log_enum.color_hsluv,
        };

        let color_id = insert_color_type.insert(&[&color_type])?;

        // Components
        match color {
            &Color::Rgba(r, g, b, _) => {
                insert_color_rgb.insert(&[
                    &color_id,
                    &(r as f64),
                    &(g as f64),
                    &(b as f64)
                ])?;
            },
            &Color::Hsluv(h, s, l, _) => {
                insert_color_hsluv.insert(&[
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