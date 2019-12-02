use super::*;
use super::db_enum::*;
use super::flo_store::*;

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Inserts a brush definition, leaving the ID on the database stack
    ///
    pub fn insert_brush(db: &mut TFile, brush_definition: &BrushDefinition) -> Result<()> {
        use self::DatabaseUpdate::*;

        match brush_definition {
            &BrushDefinition::Simple => {
                db.update(vec![
                    PushBrushType(BrushDefinitionType::from(brush_definition)),
                ])
            },

            &BrushDefinition::Ink(ref ink_defn) => {
                db.update(vec![
                    PushBrushType(BrushDefinitionType::from(brush_definition)),
                    PushInkBrush(ink_defn.min_width, ink_defn.max_width, ink_defn.scale_up_distance)
                ])
            }
        }
    }

    ///
    /// Inserts some brush properties into the database, leaving the ID on the database stack
    ///
    pub fn insert_brush_properties(db: &mut TFile, brush_properties: &BrushProperties) -> Result<()> {
        use self::DatabaseUpdate::*;

        Self::insert_color(db, &brush_properties.color)?;

        db.update(vec![
            PushBrushProperties(brush_properties.size, brush_properties.opacity)
        ])?;

        Ok(())
    }

    ///
    /// Retrieves the brush definition with the specified ID
    ///
    pub fn get_brush_definition(db: &mut TFile, brush_id: i64) -> Result<BrushDefinition> {
        use self::BrushDefinitionType::*;

        let brush_entry = db.query_brush(brush_id)?;

        match brush_entry.brush_type {
            Simple  => Ok(BrushDefinition::Simple),
            Ink     => {
                let (min_width, max_width, scale_up_distance) = brush_entry.ink_defn.unwrap_or((0.0, 0.0, 0.0));
                let min_width           = min_width as f32;
                let max_width           = max_width as f32;
                let scale_up_distance   = scale_up_distance as f32;

                Ok(BrushDefinition::Ink(InkDefinition {
                    min_width, max_width, scale_up_distance
                }))
            }
        }
    }

    ///
    /// Retrieves the brush properties with the specified ID
    ///
    pub fn get_brush_properties(db: &mut TFile, brush_properties_id: i64) -> Result<BrushProperties> {
        let brush_properties_entry  = db.query_brush_properties(brush_properties_id)?;
        let color                   = Self::get_color(db, brush_properties_entry.color_id)?;

        Ok(BrushProperties {
            size:       brush_properties_entry.size as f32,
            opacity:    brush_properties_entry.opacity as f32,
            color:      color
        })
    }
}
