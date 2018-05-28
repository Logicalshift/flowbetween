use super::*;
use super::db_enum::*;
use super::flo_store::*;

use self::DatabaseUpdate::*;

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Inserts a set of edits into the database
    /// 
    pub fn insert_edits(&mut self, edits: &[AnimationEdit]) -> Result<()> {
        // Insert all of the edits in turn
        self.db.begin_queuing();
        for edit in edits {
            self.insert_edit_log(edit)?;
        }
        self.db.execute_queue()?;

        Ok(())
    }

    ///
    /// Inserts an element ID into the edit log
    /// 
    fn insert_element_id(db: &mut TFile, element_id: &ElementId) -> Result<()> {
        use self::ElementId::*;

        match element_id {
            &Unassigned     => { },
            &Assigned(id)   => { db.update(vec![PushEditLogElementId(id)])?; },
        }

        Ok(())
    }

    ///
    /// Inserts a single AnimationEdit into the edit log
    /// 
    fn insert_edit_log<'a>(&mut self, edit: &AnimationEdit) -> Result<()> {
        // Create the edit type and push the ID
        self.db.update(vec![PushEditType(EditLogType::from(edit))])?;

        // Insert the values for this edit and pop the ID
        self.insert_animation_edit(edit)?;

        Ok(())
    }

    ///
    /// Inserts the values for an AnimationEdit into the edit log (db must have an edit ID pushed. This will be popped when this returns)
    /// 
    fn insert_animation_edit<'a>(&mut self, edit: &AnimationEdit) -> Result<()> {
        use animation::AnimationEdit::*;

        match edit {
            &Layer(layer_id, ref layer_edit)            => { 
                self.db.update(vec![PushEditLogLayer(layer_id)])?;
                self.insert_layer_edit(layer_edit)?;
            },

            &SetSize(width, height)                         => { 
                self.db.update(vec![PopEditLogSetSize(width as f32, height as f32)])?;
            },

            &AddNewLayer(layer_id)                          => {
                self.db.update(vec![PushEditLogLayer(layer_id), Pop])?;
            },

            &RemoveLayer(layer_id)                          => {
                self.db.update(vec![PushEditLogLayer(layer_id), Pop])?;
            }

            &Element(element_id, when, ref element_edit)    => {
                Self::insert_element_id(&mut self.db, &element_id)?;
                self.db.update(vec![PushEditLogWhen(when)])?;
                self.insert_element_edit(element_edit)?;
            }
        };

        Ok(())
    }

    ///
    /// Inserts the parameters for an element edit into the edit log
    /// 
    fn insert_element_edit(&mut self, edit: &ElementEdit) -> Result<()> {
        Ok(())
    }

    ///
    /// Inserts the values for a LayerEdit into the edit log (db must have an edit ID pushed. This will be popped when this returns)
    /// 
    fn insert_layer_edit(&mut self, edit: &LayerEdit) -> Result<()> {
        use animation::LayerEdit::*;

        match edit {
            &Paint(when, ref paint_edit)    => {
                self.db.update(vec![PushEditLogWhen(when)])?;
                self.insert_paint_edit(paint_edit)?;
            }

            &AddKeyFrame(when)              => {
                self.db.update(vec![PushEditLogWhen(when), Pop])?;
            }

            &RemoveKeyFrame(when)           => {
                self.db.update(vec![PushEditLogWhen(when), Pop])?;
            }
        }

        Ok(())
    }

    ///
    /// Inserts the values for a LayerEdit into the edit log (db must have an edit ID pushed. This will be popped when this returns)
    /// 
    fn insert_paint_edit<'a>(&mut self, edit: &PaintEdit) -> Result<()> {
        use animation::PaintEdit::*;

        match edit {
            &SelectBrush(ref id, ref definition, ref drawing_style) => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush(&mut self.db, definition)?;
                self.db.update(vec![PopEditLogBrush(DrawingStyleType::from(drawing_style))])?;
            },

            &BrushProperties(ref id, ref properties)                => {
                Self::insert_element_id(&mut self.db, id)?;
                Self::insert_brush_properties(&mut self.db, properties)?;
                self.db.update(vec![PopEditLogBrushProperties])?;
            },

            &BrushStroke(ref id, ref points)                        => {
                Self::insert_element_id(&mut self.db, id)?;
                self.db.update(vec![PushRawPoints(Arc::clone(points)), Pop])?;
            }
        }

        Ok(())
    }
}
