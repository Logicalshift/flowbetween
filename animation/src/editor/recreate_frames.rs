use super::element_wrapper::*;
use crate::undo::*;
use crate::traits::*;
use crate::storage::*;

use futures::prelude::*;
use flo_stream::*;

use std::iter;
use std::time::{Duration};

impl ReversedEdits {
    ///
    /// Returns the edits required to regenerate an entire keyframe
    ///
    pub (crate) fn with_recreated_keyframe<'a>(layer_id: u64, keyframe: Duration, storage_requests: &'a mut Publisher<Vec<StorageCommand>>, storage_responses: &'a mut impl Stream<Item=Vec<StorageResponse>>) -> impl 'a + Future<Output=ReversedEdits> {
        async move {
            unimplemented!()
        }
    }
}
