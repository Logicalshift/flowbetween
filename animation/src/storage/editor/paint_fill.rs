use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::super::traits::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Provides an implementaton of the fill operation
    ///
    pub (super) fn paint_fill<'a>(&'a mut self, layer_id: u64, when: Duration, path_id: ElementId, point: RawPoint, options: &Vec<FillOption>) -> impl 'a+Future<Output=Option<ElementWrapper>> {
        async move {
            None
        }
    }
}
