use super::*;
use super::db_enum::*;
use super::flo_store::*;

use std::collections::*;
use std::time::Duration;

///
/// Provides an API for performing edits on a database
/// 
pub struct AnimationEditor<TFile: FloFile+Send> {
    /// The core, where the edits are sent
    core: Arc<Desync<AnimationDbCore<TFile>>>,
}

impl<TFile: FloFile+Send+'static> AnimationEditor<TFile> {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore<TFile>>>) -> AnimationEditor<TFile> {
        AnimationEditor {
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

    ///
    /// Performs a set of edits on this item
    /// 
    pub fn perform_edits(&mut self, edits: Vec<AnimationEdit>) {
        self.core.async(move |core| {
            edits.into_iter().for_each(move |edit| core.perform_edit(edit));
        });
    }
}
