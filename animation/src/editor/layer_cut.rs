use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::traits::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_curves::bezier::path::algorithms::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

///
/// The output of the layer cut operation
///
pub (super) struct LayerCut {
    /// The group of elements that are outside of the path
    pub outside_path: Option<ElementWrapper>,

    /// The group of elements that are inside of the path
    pub inside_path: Option<ElementWrapper>,

    /// The elements that should be removed from the layer and replaced with the inside/outside groups
    pub replaced_elements: Vec<ElementId>
}

impl StreamAnimationCore {
    ///
    /// Splits all the elements in layer that intersect a supplied path into two groups, returning the groups to add and the elements 
    /// to remove in order to perform the split
    ///
    pub (super) fn layer_cut<'a>(&'a mut self, layer_id: u64, when: Duration, path_components: Arc<Vec<PathComponent>>) -> impl 'a+Future<Output=LayerCut> {
        async move {
            // Change the path components into a Path
            let path    = Path::from_elements(path_components.iter().cloned());
            let bounds  = Rect::from(&path);

            LayerCut {
                outside_path:       None,
                inside_path:        None,
                replaced_elements:  vec![]
            }
        }
    }
}
