use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::layer_properties::*;
use super::super::super::traits::*;

use ::desync::*;
use flo_stream::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;
use std::ops::{Range};
use std::time::{Duration};

struct StreamAnimationCore<StorageResponseStream> {
    /// Stream where responses to the storage requests are sent
    storage_responses: StorageResponseStream,

    /// Publisher where we can send requests for storage actions
    storage_requests: Publisher<Vec<StorageCommand>>
}

///
/// Animation that sends its updates to a storage stream
///
pub struct StreamAnimation<StorageResponseStream: Send+Unpin> {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore<StorageResponseStream>>>
}

impl<StorageResponseStream> StreamAnimation<StorageResponseStream>
where StorageResponseStream: 'static+Send+Unpin+Stream<Item=Vec<StorageResponse>> {
    ///
    /// Creates a new stream animation. The result is the animation implementation and the
    /// stream of requests to be sent to the storage layer
    ///
    pub fn new(storage_responses: StorageResponseStream) -> (StreamAnimation<StorageResponseStream>, impl Stream<Item=Vec<StorageCommand>>+Unpin) {
        // Create the storage requests. When the storage layer is running behind, we'll buffer up to 10 of these
        let mut requests    = Publisher::new(10);
        let commands        = requests.subscribe();

        // The core is used to actually execute the requests
        let core            = StreamAnimationCore {
            storage_responses:  storage_responses,
            storage_requests:   requests
        };

        // Build the animation
        let animation       = StreamAnimation {
            core: Arc::new(Desync::new(core))
        };

        // Result is the animation and the command stream
        (animation, commands)
    }
}

impl<StorageResponseStream> Animation for StreamAnimation<StorageResponseStream>
where StorageResponseStream: 'static+Send+Unpin+Stream<Item=Vec<StorageResponse>> {
    ///
    /// Retrieves the frame size of this animation
    ///
    fn size(&self) -> (f64, f64) {
        unimplemented!()
    }

    ///
    /// Retrieves the length of this animation
    ///
    fn duration(&self) -> Duration {
        unimplemented!()
    }

    ///
    /// Retrieves the duration of a single frame
    ///
    fn frame_length(&self) -> Duration {
        unimplemented!()
    }

    ///
    /// Retrieves the IDs of the layers in this object
    ///
    fn get_layer_ids(&self) -> Vec<u64> {
        unimplemented!()
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    ///
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<dyn Layer>> {
        unimplemented!()
    }

    ///
    /// Retrieves the total number of items that have been performed on this animation
    ///
    fn get_num_edits(&self) -> usize {
        unimplemented!()
    }

    ///
    /// Reads from the edit log for this animation
    ///
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> BoxStream<'a, AnimationEdit> {
        unimplemented!()
    }

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    ///
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        unimplemented!()
    }
}

impl<StorageResponseStream> EditableAnimation for StreamAnimation<StorageResponseStream>
where StorageResponseStream: 'static+Send+Unpin+Stream<Item=Vec<StorageResponse>> {
    ///
    /// Retrieves a sink that can be used to send edits for this animation
    ///
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    ///
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        unimplemented!()
    }

    ///
    /// Sends a set of edits straight to this animation
    /// 
    /// (Note that these are not always published to the publisher)
    ///
    fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        unimplemented!()
    }
}
