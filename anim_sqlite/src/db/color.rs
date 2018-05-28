use super::*;
use super::db_enum::*;
use super::flo_store::*;

use canvas::*;

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Inserts a colour definition, leaving the ID on the database stack
    /// 
    pub fn insert_color(db: &mut TFile, color: &Color) -> Result<()> {
        use self::DatabaseUpdate::*;

        match color {
            &Color::Rgba(r, g, b, _) => {
                db.update(vec![
                    PushColorType(ColorType::Rgb),
                    PushRgb(r, g, b)
                ])
            },

            &Color::Hsluv(h, s, l, _) => {
                db.update(vec![
                    PushColorType(ColorType::Hsluv),
                    PushHsluv(h, s, l)
                ])
            },
        }
    }

    ///
    /// Decodes the colour with the specified ID from the database
    /// 
    pub fn get_color(db: &mut TFile, color_id: i64) -> Result<Color> {
        use self::ColorType::*;

        let entry = db.query_color(color_id)?;

        match entry.color_type {
            Rgb     => {
                let (r, g, b) = entry.rgb.unwrap_or((0.0, 0.0, 0.0));
                Ok(Color::Rgba(r as f32, g as f32, b as f32, 1.0))
            },

            Hsluv   => {
                let (h, s, l) = entry.hsluv.unwrap_or((0.0, 0.0, 0.0));
                Ok(Color::Hsluv(h as f32, s as f32, l as f32, 1.0))
            }
        }
    }
}
