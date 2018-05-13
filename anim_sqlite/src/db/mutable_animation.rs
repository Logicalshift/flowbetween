use super::*;
use super::db_enum::*;
use super::flo_store::*;

use std::collections::*;
use std::time::Duration;

///
/// Class used for the animation object for a database
/// 
pub struct AnimationDbEditor<TFile: FloFile+Send> {
    /// The core, where the edits are sent
    core: Arc<Desync<AnimationDbCore<TFile>>>,

    /// The layers created by this editor (used to track state)
    layers: HashMap<u64, SqliteVectorLayer<TFile>>
}

impl<TFile: FloFile+Send+'static> AnimationDbEditor<TFile> {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore<TFile>>>) -> AnimationDbEditor<TFile> {
        AnimationDbEditor {
            core:   Arc::clone(core),
            layers: HashMap::new()
        }
    }

    ///
    /// Performs an edit on this item (if the core's error condition is clear)
    /// 
    fn edit<TEdit: Fn(&mut TFile) -> Result<()>+Send+'static>(&mut self, edit: TEdit) {
        self.core.async(move |core| core.edit(edit))
    }
}

impl<TFile: FloFile+Send+'static> MutableAnimation for AnimationDbEditor<TFile> {
    fn set_size(&mut self, size: (f64, f64)) {
        // Update the size for the current animation
        self.edit(move |db| {
            db.update(vec![
                DatabaseUpdate::UpdateCanvasSize(size.0, size.1)
            ])?;

            Ok(())
        })
    }

    fn add_layer(&mut self, new_layer_id: u64) {
        // Create a layer with this assigned ID
        self.edit(move |db| {
            db.update(vec![
                DatabaseUpdate::PushLayerType(LayerType::Vector),
                DatabaseUpdate::PushAssignLayer(new_layer_id),
                DatabaseUpdate::Pop
            ])
        })
    }

    fn remove_layer(&mut self, old_layer_id: u64) {
        // Remove the cached version of this layer
        self.layers.remove(&old_layer_id);

        // Create a layer with this assigned ID
        self.edit(move |db| {
            db.update(vec![
                DatabaseUpdate::PushLayerForAssignedId(old_layer_id),
                DatabaseUpdate::PopDeleteLayer
            ])
        })
    }

    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
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

        // Edit it
        layer.map(|layer| Editor::new(layer as &mut Layer))
    }

    fn edit_element(&mut self, element_id: ElementId, when: Duration, edit: ElementEdit) {
        unimplemented!()
    }
}
