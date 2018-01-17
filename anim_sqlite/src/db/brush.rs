use super::*;
use super::db_enum::*;
use super::db_update::*;

impl AnimationDbCore {
    ///
    /// Inserts a brush definition, leaving the ID on the database stack
    /// 
    pub fn insert_brush(&mut self, brush_definition: &BrushDefinition) -> Result<()> {
        use self::DatabaseUpdate::*;

        match brush_definition {
            &BrushDefinition::Simple => {
                self.db.update(vec![
                    PushBrushType(BrushDefinitionType::from(brush_definition)),
                ])
            },

            &BrushDefinition::Ink(ref ink_defn) => {
                self.db.update(vec![
                    PushBrushType(BrushDefinitionType::from(brush_definition)),
                    PushInkBrush(ink_defn.min_width, ink_defn.max_width, ink_defn.scale_up_distance)
                ])
            }
        }
    }

    ///
    /// Inserts some brush properties into the database, leaving the ID on the database stack
    ///
    pub fn insert_brush_properties(&mut self, brush_properties: &BrushProperties) -> Result<()> {
        use self::DatabaseUpdate::*;

        self.db.update(vec![
            PushBrushProperties(brush_properties.size, brush_properties.opacity)
        ])
    }
}