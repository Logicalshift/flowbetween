use super::db_enum::*;
use super::flo_store::*;
use super::vector_layer::*;

use animation::*;

use rusqlite::*;
use std::collections::HashMap;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore<TFile: FloFile+Send> {
    /// The database connection
    pub db: TFile,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,

    /// The layers created by this editor (used to track state)
    layers: HashMap<u64, SqliteVectorLayer<TFile>>
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Performs an edit on this core if the failure condition is clear
    /// 
    pub fn edit<TEdit: FnOnce(&mut TFile) -> Result<()>>(&mut self, edit: TEdit) {
        // Perform the edit if there is no failure
        if self.failure.is_none() {
            self.failure = edit(&mut self.db).err();
        }
    }

    ///
    /// Performs an edit on this core
    /// 
    pub fn perform_edit(&mut self, edit: AnimationEdit) {
        use self::AnimationEdit::*;

        match edit {
            SetSize(width, height) => {
                self.db.update(vec![
                    DatabaseUpdate::UpdateCanvasSize(width, height)
                ])?;
            },

            AddNewLayer(new_layer_id) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerType(LayerType::Vector),
                    DatabaseUpdate::PushAssignLayer(new_layer_id),
                    DatabaseUpdate::Pop
                ])?;
            },

            RemoveLayer(old_layer_id) => {
                // Remove the cached version of this layer
                self.layers.remove(&old_layer_id);

                // Create a layer with this assigned ID
                self.db.update(vec![
                    DatabaseUpdate::PushLayerForAssignedId(old_layer_id),
                    DatabaseUpdate::PopDeleteLayer
                ])?
            },

            Layer(layer_id, layer_edit) => {
                // Create the layer if one is not already cached
                let layer = if !self.layers.contains_key(&layer_id) {
                    let new_layer = SqliteVectorLayer::from_assigned_id(&self.core, layer_id);

                    if let Some(new_layer) = new_layer {
                        self.layers.insert(layer_id, new_layer);
                        self.layers.get_mut(&layer_id)
                    } else {
                        None
                    }
                } else {
                    self.layers.get_mut(&layer_id)
                };

                // Edit the layer
                layer.edit(&mut self.db, layer_edit);
            },

            Element(id, when, edit) => {
                unimplemented!()
            }
        }
    }
}
