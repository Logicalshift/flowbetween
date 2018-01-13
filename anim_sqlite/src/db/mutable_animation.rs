use super::*;

use std::mem;

///
/// Class used for the animation object for a database
/// 
pub struct AnimationEditor {
    /// The core, where the edits are sent
    core: Arc<Desync<AnimationDbCore>>,

    /// The edits to perform on the database core when this editor is done with
    edits: Vec<Box<Fn(&Connection, i64) -> Result<()>+Send>>
}

impl AnimationEditor {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore>>) -> AnimationEditor {
        AnimationEditor {
            core:   Arc::clone(core),
            edits:  vec![]
        }
    }
}

impl Drop for AnimationEditor {
    fn drop(&mut self) {
        // Grab the set of uncommitted edits
        let mut edits = vec![];
        mem::swap(&mut self.edits, &mut edits);

        // Send to the core
        self.core.async(move |core| {
            let failure = &mut core.failure;

            for edit in edits {
                if failure.is_none() {
                    *failure = (edit)(&core.sqlite, core.animation_id).err()
                }
            }
        })
    }
}

impl MutableAnimation for AnimationEditor {
    fn set_size(&mut self, size: (f64, f64)) {
        // Update the size for the current animation
        self.edits.push(Box::new(move |sqlite, animation_id| {
            sqlite.execute(
                "UPDATE Flo_Animation SET SizeX = ?, SizeY = ? WHERE AnimationId = ?",
                &[&size.0, &size.1, &animation_id]
            )?;

            Ok(())
        }))
    }

    fn add_layer(&mut self, new_layer_id: u64) {
        // Create a layer with this assigned ID
        self.edits.push(Box::new(move |sqlite, animation_id| {
            // TODO: hard codes the layer type as 0 (vector layer), but we can't set layer types right now anyway
            // Create the layer
            let layer_id: i32 = sqlite.execute(
                "INSERT INTO Flo_LayerType (LayerType) VALUES (0)",
                &[]
            )?;

            // Give it an assigned ID
            sqlite.execute(
                "INSERT INTO Flo_AnimationLayers (AnimationId, LayerId, AssignedLayerId) VALUES (?, ?, ?)",
                &[&animation_id, &layer_id, &(new_layer_id as i64)]
            )?;

            Ok(())
        }))
    }

    fn remove_layer(&mut self, old_layer_id: u64) {
        unimplemented!()
    }

    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        unimplemented!()
    }
}
