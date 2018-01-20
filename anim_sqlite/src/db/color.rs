use super::*;
use super::db_enum::*;
use super::flo_store::*;

use canvas::*;

impl<TFile: FloFile> AnimationDbCore<TFile> {
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
}