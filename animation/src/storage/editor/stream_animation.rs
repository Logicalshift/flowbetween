use super::stream_layer::*;
use super::stream_animation_core::*;
use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::layer_properties::*;
use super::super::super::traits::*;

use ::desync::*;
use flo_stream::*;

use futures::prelude::*;
use futures::task::{Poll};
use futures::stream;
use futures::stream::{BoxStream};

use std::sync::*;
use std::ops::{Range};
use std::time::{Duration};

///
/// Animation that sends its updates to a storage stream
///
pub struct StreamAnimation {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore>>,

    /// The publisher for the edits to this animation
    edit_publisher: Publisher<Arc<Vec<AnimationEdit>>>,

    /// Available synchronous requests
    idle_sync_requests: Desync<Vec<Desync<Option<Vec<StorageResponse>>>>>,

    /// The properties of the animation
    file_properties: Desync<Option<FileProperties>>
}

impl StreamAnimation {
    ///
    /// Creates a new stream animation. The result is the animation implementation and the
    /// stream of requests to be sent to the storage layer
    ///
    pub fn new<ConnectStream: FnOnce(BoxStream<'static, Vec<StorageCommand>>) -> BoxStream<'static, Vec<StorageResponse>>>(connect_stream: ConnectStream) -> StreamAnimation {
        // Create the storage requests. When the storage layer is running behind, we'll buffer up to 10 of these
        let mut requests        = Publisher::new(10);
        let commands            = requests.subscribe().boxed();
        let storage_responses   = connect_stream(commands);
        let mut edit_publisher  = Publisher::new(10);

        // The core is used to actually execute the requests
        let core            = StreamAnimationCore {
            storage_responses:  storage_responses,
            storage_requests:   requests,
            next_element_id:    None,
            cached_keyframe:    None,
            path_brush_defn:    None,
            path_brush_props:   None
        };
        let core            = Arc::new(Desync::new(core));

        // Anything published to the editor is piped into the core
        pipe_in(Arc::clone(&core), edit_publisher.subscribe(), |core, edits| {
            async move {
                core.perform_edits(edits).await;
            }.boxed()
        });

        // Build the animation
        StreamAnimation {
            core:               core,
            idle_sync_requests: Desync::new(vec![]),
            file_properties:    Desync::new(None),
            edit_publisher:     edit_publisher
        }
    }

    ///
    /// Raises a panic if this animation has reached an error state
    ///
    pub fn panic_on_error(&self) {

    }

    ///
    /// Performs an asynchronous request on a storage layer for this animation
    ///
    pub fn request_async(&self, request: Vec<StorageCommand>) -> impl Future<Output=Option<Vec<StorageResponse>>> {
        self.core.future(move |core| {
            async move {
                core.storage_requests.publish(request).await;
                core.storage_responses.next().await
            }.boxed()
        }).map(|res| {
            res.unwrap_or(None)
        })
    }

    ///
    /// Performs a synchronous request on the storage layer for this animation
    /// 
    /// Synchronous requests are fairly slow, so should be avoided in inner loops
    ///
    pub fn request_sync(&self, request: Vec<StorageCommand>) -> Option<Vec<StorageResponse>> {
        // Fetch a copy of the core
        let core = Arc::clone(&self.core);

        // Get an idle sync request desync
        //   We use desync instead of the futures executor as the executor will panic if we are called from within another future
        //   (desync provides a way around this problem)
        let sync_request = self.idle_sync_requests.sync(|reqs| {
            let next_request = reqs.pop();
            if let Some(next_request) = next_request {
                next_request
            } else {
                let req = Desync::new(None);
                req
            }
        });

        // Queue a request
        let _ = sync_request.future(move |data| {
            async move {
                let result = core.future(|core| {
                    async move {
                        core.storage_requests.publish(request).await;
                        core.storage_responses.next().await
                    }.boxed()
                }).await;

                *data = result.unwrap_or(None);
            }.boxed()
        });

        // Retrieve the result
        let result = sync_request.sync(|req| req.take());

        // Return the sync_request to the pool
        self.idle_sync_requests.desync(move |reqs| { reqs.push(sync_request) });

        // Return the result of the request
        result
    }

    ///
    /// Retrieves the current file properties for the animation
    ///
    fn file_properties(&self) -> FileProperties {
        let properties = self.file_properties.sync(|props| props.clone());

        let properties = if let Some(properties) = properties {
            // Already got the properties
            properties
        } else {
            // Retrieve the properties from storage (and update the version we have stored if there is one)
            let mut response = self.request_sync(vec![StorageCommand::ReadAnimationProperties]).unwrap_or_else(|| vec![]);
            let properties;

            match response.pop() {
                Some(StorageResponse::NotFound) => {
                    // File properties are not set
                    properties = FileProperties::default();
                    self.file_properties.sync(|props| { *props = Some(properties.clone()); })
                }

                Some(StorageResponse::AnimationProperties(props)) => {
                    // Deserialize the file properties
                    properties = FileProperties::deserialize(&mut props.chars()).expect("Could not parse file properties");
                    self.file_properties.sync(|props| { *props = Some(properties.clone()); })
                }

                _ => panic!("Unexpected response while reading file properties")
            }

            properties
        };

        // Return the file properties
        properties
    }
}

impl Animation for StreamAnimation {
    ///
    /// Retrieves the frame size of this animation
    ///
    fn size(&self) -> (f64, f64) {
        self.file_properties().size
    }

    ///
    /// Retrieves the length of this animation
    ///
    fn duration(&self) -> Duration {
        self.file_properties().duration
    }

    ///
    /// Retrieves the duration of a single frame
    ///
    fn frame_length(&self) -> Duration {
        self.file_properties().frame_length
    }

    ///
    /// Retrieves the IDs of the layers in this object
    ///
    fn get_layer_ids(&self) -> Vec<u64> {
        let layer_responses = self.request_sync(vec![StorageCommand::ReadLayers]).unwrap_or_else(|| vec![]);

        layer_responses
            .into_iter()
            .map(|response| {
                match response {
                    StorageResponse::LayerProperties(id, _) => Some(id),
                    _                                       => None
                }
            })
            .flatten()
            .collect()
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    ///
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<dyn Layer>> {
        // Read the properties for the specified layer
        let layer_properties = self.request_sync(vec![StorageCommand::ReadLayerProperties(layer_id)]);

        if let Some(StorageResponse::LayerProperties(_, serialized)) = layer_properties.and_then(|mut props| props.pop()) {
            if let Some(layer_properties) = LayerProperties::deserialize(&mut serialized.chars()) {
                // Found the layer
                Some(Arc::new(StreamLayer::new(Arc::clone(&self.core), layer_id, layer_properties)))
            } else {
                // Can't deserialize the layer properties
                None
            }
        } else {
            // Layer does not exist
            None
        }
    }

    ///
    /// Retrieves the total number of edits that have been performed on this animation
    ///
    fn get_num_edits(&self) -> usize {
        let mut response = self.request_sync(vec![StorageCommand::ReadEditLogLength]).unwrap_or_else(|| vec![]);

        match response.pop() {
            Some(StorageResponse::NumberOfEdits(num_edits)) => num_edits,

            _ => panic!("Unexpected response while reading number of edits")
        }
    }

    ///
    /// Reads from the edit log for this animation
    ///
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> BoxStream<'a, AnimationEdit> {
        // Clamp the range of edits to the maximum number of edits
        let max_edit    = self.get_num_edits();
        let range       = if range.end > max_edit {
            range.start..max_edit
        } else {
            range
        };

        // Generate a stream to read from the edit log as we go
        let per_request         = 20;
        let mut remaining       = range;
        let mut fetched         = vec![];
        let mut next_response   = None;

        stream::poll_fn(move |context| {
            loop {
                if remaining.len() != 0 && fetched.len() == 0 && next_response.is_none() {
                    // Fetch up to per_request items for each request
                    let num_to_fetch    = remaining.len();
                    let num_to_fetch    = if num_to_fetch > per_request { per_request } else { num_to_fetch };
                    let fetch_range     = (remaining.start)..(remaining.start + num_to_fetch);

                    // Start polling for the next batch
                    next_response       = Some(self.request_async(vec![StorageCommand::ReadEdits(fetch_range)]));
                    remaining           = (remaining.start+num_to_fetch)..(remaining.end);
                }

                if let Some(next) = fetched.pop() {
                    // Just returning the batch we've already fetched
                    return Poll::Ready(Some(next));
                } else if let Some(mut waiting) = next_response.take() {
                    // Try to retrieve the next item from the batch
                    let poll_response = waiting.poll_unpin(context);

                    match poll_response {
                        Poll::Pending           => {
                            // Keep waiting for the response
                            next_response = Some(waiting);
                            return Poll::Pending
                        },

                        Poll::Ready(response)   => {
                            // Load the edits into the fetched array
                            let mut response = response.unwrap_or(vec![]);

                            while let Some(response) = response.pop() {
                                // Ignore everything that's not an edit (we have no way to do error handling here)
                                if let StorageResponse::Edit(_num, serialized_edit) = response {
                                    // Store edits that deserialize successfully on the fetched list
                                    if let Some(edit) = AnimationEdit::deserialize(&mut serialized_edit.chars()) {
                                        fetched.push(edit)
                                    }
                                }
                            }
                        }
                    }

                } else if remaining.len() == 0 {
                    // Reached the end of the stream
                    return Poll::Ready(None);
                }
            }
        }).fuse().boxed()
    }

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    ///
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        unimplemented!()
    }
}

impl EditableAnimation for StreamAnimation {
    ///
    /// Retrieves a sink that can be used to send edits for this animation
    ///
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    ///
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.edit_publisher.republish()
    }

    ///
    /// Sends a set of edits straight to this animation
    /// 
    /// (Note that these are not always published to the publisher)
    ///
    fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        // Get a publisher to send the edits to (this editor does send its edits to the publisher)
        let mut publisher = self.edit_publisher.republish();

        // Get an idle sync request desync
        //   We use desync instead of the futures executor as the executor will panic if we are called from within another future
        //   (desync provides a way around this problem)
        let sync_request = self.idle_sync_requests.sync(|reqs| {
            let next_request = reqs.pop();
            if let Some(next_request) = next_request {
                next_request
            } else {
                let req = Desync::new(None);
                req
            }
        });

        // Queue a request
        let _ = sync_request.future(move |_| {
            async move {
                publisher.publish(Arc::new(edits)).await
            }.boxed()
        });

        // Return the sync_request to the pool
        self.idle_sync_requests.desync(move |reqs| { reqs.push(sync_request) });
    }
}
