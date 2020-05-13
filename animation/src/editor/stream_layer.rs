use super::stream_frame::*;
use super::stream_layer_cache::*;
use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::storage::layer_properties::*;
use crate::traits::*;

use ::desync::*;
use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::ops::{Range, Deref};

///
/// A layer from a stream animation
///
pub struct StreamLayer {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore>>,

    /// The ID of the layer that this should fetch
    layer_id: u64,

    /// The properties for this layer
    properties: LayerProperties,

    /// Available synchronous requests
    idle_sync_requests: Desync<Vec<Desync<Option<Vec<StorageResponse>>>>>,
}

impl StreamLayer {
    ///
    /// Creates a new stream layer from a core, a layer ID and some layer properties
    ///
    pub (super) fn new(core: Arc<Desync<StreamAnimationCore>>, layer_id: u64, properties: LayerProperties) -> StreamLayer {
        StreamLayer {
            core:               core,
            layer_id:           layer_id,
            properties:         properties,
            idle_sync_requests: Desync::new(vec![])
        }
    }

    ///
    /// Performs a synchronous request on the storage layer for this animation
    /// 
    /// Synchronous requests are fairly slow, so should be avoided in inner loops
    ///
    fn request_sync(&self, request: Vec<StorageCommand>) -> Option<Vec<StorageResponse>> {
        request_core_sync(Arc::clone(&self.core), &self.idle_sync_requests, request)
    }
}

impl Layer for StreamLayer {
    ///
    /// The ID associated with this layer
    ///
    fn id(&self) -> u64 {
        self.layer_id
    }

    ///
    /// Retrieves the name associated with this layer (or none if no name has been assigned yet)
    ///
    fn name(&self) -> Option<String> {
        Some(self.properties.name.clone())
    }

    ///
    /// The types of edit that are supported by this layer
    ///
    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![
            LayerEditType::Vector
        ]
    }

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index: Duration) -> Arc<dyn Frame> {
        // Retrieve the keyframe from the core
        let core            = Arc::clone(&self.core);
        let layer_id        = self.layer_id;
        let keyframe_core   = Desync::new(None);

        // Load into the keyframe_core desync
        let _               = keyframe_core.future(move |frame| {
            async move {
                *frame = core.future(move |core| {
                    async move {
                        core.load_keyframe(layer_id, time_index).await
                    }.boxed()
                }).await.unwrap_or(None);
            }.boxed()
        });

        // Retrieve the result when the future completes
        let keyframe_core   = keyframe_core.sync(|frame| frame.take());

        // Create a frame with the keyframe core
        Arc::new(StreamFrame::new(time_index, keyframe_core))
    }

    ///
    /// Retrieves the times where key frames exist during a specified time range
    ///
    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<dyn Iterator<Item=Duration>> {
        // Request the keyframe locations from the storage
        let key_frames = self.request_sync(vec![StorageCommand::ReadKeyFrames(self.layer_id, when.clone())]);

        // Return the keyframes that start within the specified range
        Box::new(key_frames.unwrap_or_else(|| vec![])
            .into_iter()
            .filter_map(move |response| {
                match response {
                    StorageResponse::KeyFrame(start, _end)  => if start >= when.start && start < when.end { Some(start) } else { None },
                    _                                       => None
                }
            }))
    }

    ///
    /// Retrieves the previous and next keyframes from a particular point in time
    ///
    /// (If there's a keyframe at this point in time, it is not returned)
    ///
    fn previous_and_next_key_frame(&self, when: Duration) -> (Option<Duration>, Option<Duration>) {
        // Request the keyframe locations from the storage
        let start_time          = if when > Duration::from_nanos(0) { when - Duration::from_nanos(1) } else { when };
        let range               = (start_time)..(when + Duration::from_micros(1));
        let key_frames          = self.request_sync(vec![StorageCommand::ReadKeyFrames(self.layer_id, range)]);

        // We need to get the highest key frame before the 'when' time and the lowest after the 'when' time
        let mut highest_before  = None;
        let mut lowest_after    = None;

        let times               = key_frames.unwrap_or_else(|| vec![])
            .into_iter()
            .map(|response| match response {
                StorageResponse::KeyFrame(start, end)       => vec![start, end],
                StorageResponse::NotInAFrame(next_frame)    => vec![next_frame],
                _                                           => vec![]
            })
            .flatten();

        for frame_time in times {
            // Try to populate with this frame if the 'before' time is not set yet
            if highest_before.is_none() && frame_time < when {
                highest_before = Some(frame_time);
            }

            // Use this frame as the 'before' frame if it's closer to the 'when' time
            highest_before = highest_before
                .map(|before| {
                    if frame_time < when && frame_time > before {
                        frame_time
                    } else {
                        before
                    }
                });

            // Try to populate with this frame if the 'after' time is not set yet
            if lowest_after.is_none() {
                if frame_time > when {
                    lowest_after = Some(frame_time);
                }
            }

            // Use this frame as the 'after' frame if it's closer to the 'when' time
            lowest_after = lowest_after
                .map(|after| {
                    if frame_time > when && frame_time < after {
                        frame_time
                    } else {
                        after
                    }
                });
        }

        // End of the range indicated by i64::max_value() microseconds
        if lowest_after == Some(Duration::from_micros(i64::max_value() as u64)) { 
            lowest_after = None;
        }

        // Keyframes are at the highest time we got before the 'when' time and the lowest time after
        (highest_before, lowest_after)
    }

    ///
    /// Retrieves the definition of this layer as a vector layer
    ///
    fn as_vector_layer<'a>(&'a self) -> Option<Box<dyn 'a+Deref<Target=dyn 'a+VectorLayer>>> {
        let as_vector_layer: &dyn VectorLayer = self;

        Some(Box::new(as_vector_layer))
    }

    ///
    /// Retrieves the canvas cache at the specified time
    ///
    fn get_canvas_cache_at_time(&self, time_index: Duration) -> Arc<dyn CanvasCache> {
        Arc::new(StreamLayerCache::new(Arc::clone(&self.core), self.layer_id, time_index))
    }
}

impl VectorLayer for StreamLayer {
    ///
    /// The brush that will be active for the next element that's added to this layer (if one is set)
    ///
    fn active_brush(&self, when: Duration) -> Option<Arc<dyn Brush>> {
        // Create a desync to store the result, once we have it
        let result      = Desync::new(None);
        let layer_id    = self.layer_id;
        let core        = Arc::clone(&self.core);

        // Start loading the result into the desync
        let _ = result.future(move |result| {
            async move {
                // Store the result from the core
                *result = core.future(move |core| {
                    async move {
                        // Fetch the keyframe at this time
                        let frame = core.edit_keyframe(layer_id, when).await?;

                        // The last brush stroke element in the frame will indicate the active brush
                        let last_brush_properties = frame.future(move |frame| {
                            async move {
                                // Fetch the last brush stroke from the frame
                                let last_brush_stroke = frame.vector_elements(when)
                                    .filter(|elem| match elem { Vector::BrushStroke(_) => true, _ => false })
                                    .last()?;

                                Some(frame.apply_properties_for_element(&last_brush_stroke, Arc::new(VectorProperties::default()), when))
                            }.boxed()
                        }).await.unwrap()?;

                        // Get the brush from the properties
                        let brush = Arc::clone(&last_brush_properties.brush);

                        Some(brush)
                    }.boxed()
                }).await.unwrap();
            }.boxed()
        });

        // Retrieve the result
        result.sync(|res| res.take())
    }
}
