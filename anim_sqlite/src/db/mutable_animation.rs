use super::*;

use animation::*;

use rusqlite::*;
use std::ops::Deref;

///
/// Class used for the animation object for a database
/// 
pub struct AnimationEditor {
    /// The core, where the edits are sent
    core: Arc<Desync<AnimationDbCore>>,

    /// The edits to perform on the database core when this editor is done with
    edits: Vec<Box<Fn(&mut Connection) -> Result<()>+Send>>
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

impl MutableAnimation for AnimationEditor {
    fn set_size(&mut self, size: (f64, f64)) {
        unimplemented!()
    }

    fn add_layer(&mut self, new_layer_id: u64) {
        unimplemented!()
    }

    fn remove_layer(&mut self, old_layer_id: u64) {
        unimplemented!()
    }

    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        unimplemented!()
    }
}
