use super::super::storage_api::*;

use ::desync::*;
use flo_stream::*;

use futures::prelude::*;

use std::sync::*;

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
    pub fn new(storage_responses: StorageResponseStream) -> (StreamAnimation<StorageResponseStream>, impl Stream<Item=Vec<StorageCommand>>) {
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
