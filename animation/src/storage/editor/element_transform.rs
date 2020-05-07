use super::stream_animation_core::*;
use crate::traits::*;

use futures::prelude::*;

impl StreamAnimationCore {
    ///
    /// Applies transformations to a set of elements
    ///
    pub fn transform_elements<'a>(&'a mut self, element_ids: &'a Vec<i64>, transformations: &'a Vec<ElementTransform>) -> impl 'a+Send+Future<Output=()> {
        async move {

        }
    }
}