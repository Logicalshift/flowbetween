use super::stream_layer::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::storage::file_properties::*;
use crate::storage::layer_properties::*;
use crate::traits::*;
use crate::serializer::*;

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
            brush_defn:         None,
            brush_props:        None,
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
            edit_publisher:     edit_publisher
        }
    }

    ///
    /// Performs an asynchronous request on a storage layer for this animation
    ///
    pub (super) fn request_async<Commands: Send+IntoIterator<Item=StorageCommand>>(&self, request: Commands) -> impl Future<Output=Option<Vec<StorageResponse>>> {
        request_core_async(&self.core, request.into_iter().collect())
    }

    ///
    /// Performs a synchronous request on the storage layer for this animation
    /// 
    /// Synchronous requests are fairly slow, so should be avoided in inner loops
    ///
    pub (super) fn request_sync<Commands: Send+IntoIterator<Item=StorageCommand>>(&self, request: Commands) -> Option<Vec<StorageResponse>> {
        request_core_sync(Arc::clone(&self.core), &self.idle_sync_requests, request.into_iter().collect())
    }

    ///
    /// Waits for any pending edits on this animation to complete
    ///
    pub (super) fn wait_for_edits(&self) {
        // Force a desync to wait for the when_empty future to complete
        let when_empty = self.edit_publisher.republish().when_empty();

        // Create a desync and wait for the 'when_empty' signal to show up (indicating all the edits have been sent to the core)
        let wait_for_edits  = Desync::new(());
        let _               = wait_for_edits.future(move |_| async move { when_empty.await; }.boxed());

        // Synchronise after the future has completed
        wait_for_edits.sync(|_| { });
    }

    ///
    /// Retrieves the current file properties for the animation
    ///
    fn file_properties(&self) -> FileProperties {
        // Retrieve the properties from storage (and update the version we have stored if there is one)
        let mut response = self.request_sync(vec![StorageCommand::ReadAnimationProperties]).unwrap_or_else(|| vec![]);
        let properties;

        match response.pop() {
            Some(StorageResponse::NotFound) => {
                // File properties are not set
                properties = FileProperties::default();
            }

            Some(StorageResponse::AnimationProperties(props)) => {
                // Deserialize the file properties
                properties = FileProperties::deserialize(&mut props.chars()).expect("Could not parse file properties");
            }

            unknown => panic!("Unexpected response {:?} while reading file properties", unknown)
        }

        properties
    }
}

impl Animation for StreamAnimation {
    ///
    /// Retrieves the frame size of this animation
    ///
    fn size(&self) -> (f64, f64) {
        self.wait_for_edits();
        self.file_properties().size
    }

    ///
    /// Retrieves the length of this animation
    ///
    fn duration(&self) -> Duration {
        self.wait_for_edits();
        self.file_properties().duration
    }

    ///
    /// Retrieves the duration of a single frame
    ///
    fn frame_length(&self) -> Duration {
        self.wait_for_edits();
        self.file_properties().frame_length
    }

    ///
    /// Retrieves the IDs of the layers in this object
    ///
    fn get_layer_ids(&self) -> Vec<u64> {
        self.wait_for_edits();

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
        self.wait_for_edits();

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
        self.wait_for_edits();

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
        self.wait_for_edits();

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
        &*self
    }
}

impl EditableAnimation for StreamAnimation {
    ///
    /// Assigns a new unique ID for creating a new motion
    ///
    /// This ID will not have been used so far and will not be used again, and can be used as the ID for the MotionElement vector element.
    ///
    fn assign_element_id(&self) -> ElementId {
        // Create a queue to run the 'assign element ID' future on
        let core    = Arc::clone(&self.core);
        let request = Desync::new(None);

        // Perform the request
        let _ = request.future(|result| {
            async move {
                *result = Some(core.future(|core| core.assign_element_id(ElementId::Unassigned).boxed()).await.unwrap())
            }.boxed()
        });

        // Retrieve the result
        request.sync(|result| result.take()).unwrap()
    }

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
                // Publish the edits
                publisher.publish(Arc::new(edits)).await;

                // Wait for them to be processed
                publisher.when_empty().await;
            }.boxed()
        });

        // Wait for the edits to complete
        sync_request.sync(|_| { });

        // Return the sync_request to the pool
        self.idle_sync_requests.desync(move |reqs| { reqs.push(sync_request) });
    }

    ///
    /// Flushes any caches this might have (forces reload from data storage)
    ///
    fn flush_caches(&self) {
        self.core.desync(|core| {
            core.cached_keyframe = None;
        });
    }
}

impl AnimationMotion for StreamAnimation {
    ///
    /// Retrieves the IDs of the motions attached to a particular element
    ///
    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        let element_id      = match element_id {
            ElementId::Assigned(id) => id,
            ElementId::Unassigned   => { return vec![]; }
        };

        // Request the keyframe that contains this element
        let core            = Arc::clone(&self.core);
        let keyframe_req    = Desync::new(None);

        let _               = keyframe_req.future(move |result| {
            async move {
                *result = core.future(move |core| core.edit_keyframe_for_element(element_id).boxed()).await.unwrap();
            }.boxed()
        });
        let keyframe        = keyframe_req.sync(|result| result.take());
        let keyframe        = match keyframe {
            Some(keyframe)  => keyframe,
            None            => { return vec![]; }
        };

        // Read the attachments for the element
        keyframe.sync(move |keyframe| {
            // Read the main element
            let element     = keyframe.elements.get(&ElementId::Assigned(element_id));
            let element     = match element {
                Some(wrapper)   => wrapper,
                None            => { return vec![]; }
            };

            // Try to read all the attachments
            let motion_attachments = element.attachments.iter()
                .filter_map(|attachment_id| keyframe.elements.get(attachment_id))
                .filter_map(|attachment_wrapper| {
                    if VectorType::from(&attachment_wrapper.element) == VectorType::Motion {
                        Some(attachment_wrapper.element.id())
                    } else {
                        None
                    }
                });

            motion_attachments.collect()
        })
    }

    ///
    /// Retrieves the IDs of the elements attached to a particular motion
    ///
    fn get_elements_for_motion(&self, motion_id: ElementId) -> Vec<ElementId> {
        if let Some(motion_id) = motion_id.id() {
            // Read the wrapper for the motion
            let response = self.request_sync(vec![StorageCommand::ReadElement(motion_id)]);

            // Read the attachments from the response (resolve the elements with no dependent elements)
            response.into_iter()
                .flatten()
                .map(|response| match response {
                    StorageResponse::Element(id, data) => {
                        // Deserialize the element to get its attachments
                        let resolver = ElementWrapper::deserialize(ElementId::Assigned(id), &mut data.chars());
                        resolver.and_then(|resolver| resolver.resolve(&mut |_| None))
                    },
                    _ => None
                }).flatten()
                .map(|wrapper| wrapper.attached_to.iter().cloned().collect::<Vec<_>>())
                .flatten()
                .collect()
        } else {
            // No elements if the ID is unassigned
            vec![]
        }
    }

    ///
    /// Retrieves the motion with the specified ID
    ///
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        let element_id      = match motion_id {
            ElementId::Assigned(id) => id,
            ElementId::Unassigned   => { return None; }
        };

        // Request the keyframe that contains this element
        let core            = Arc::clone(&self.core);
        let keyframe_req    = Desync::new(None);

        let _               = keyframe_req.future(move |result| {
            async move {
                *result = core.future(move |core| core.edit_keyframe_for_element(element_id).boxed()).await.unwrap();
            }.boxed()
        });
        let keyframe        = keyframe_req.sync(|result| result.take());
        let keyframe        = match keyframe {
            Some(keyframe)  => keyframe,
            None            => { return None; }
        };

        // Try to retrieve the element
        keyframe.sync(move |keyframe| {
            // Read the main element
            let element     = match keyframe.elements.get(&ElementId::Assigned(element_id)) {
                Some(element)   => element,
                None            => { return None; }
            };

            match &element.element {
                Vector::Motion(motion)  => Some((&*motion.motion()).clone()),
                _                       => None
            }
        })
    }
}
