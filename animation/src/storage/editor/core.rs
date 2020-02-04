use super::super::storage_api::*;
use super::super::super::traits::*;

use flo_stream::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

pub (super) struct StreamAnimationCore {
    /// Stream where responses to the storage requests are sent
    pub (super) storage_responses: BoxStream<'static, Vec<StorageResponse>>,

    /// Publisher where we can send requests for storage actions
    pub (super) storage_requests: Publisher<Vec<StorageCommand>>,
}

impl StreamAnimationCore {
    ///
    /// Performs a set of edits on the core
    ///
    pub fn perform_edits<'a>(&'a mut self, edits: Arc<Vec<AnimationEdit>>) -> impl 'a+Future<Output=()> {
        async move {
            // TODO
        }
    }
}
