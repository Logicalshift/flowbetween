use super::core::*;
use super::element_wrapper::*;
use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::super::traits::*;

use flo_stream::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// The keyframe core represents the elements in a keyframe in a particular layer
///
pub (super) struct KeyFrameCore {
    /// The elements in this keyframe
    elements: HashMap<ElementId, ElementWrapper>,

    /// The first element in the keyframe
    initial_element: Option<ElementId>,

    /// The last element in the keyframe
    last_element: Option<ElementId>,

    /// The start time of this keyframe
    start: Duration,

    /// The end time of this keyframe
    end: Duration
}

impl KeyFrameCore {
    ///
    /// Generates a keyframe by querying the animation core
    ///
    pub fn from_keyframe<'a>(core: &'a mut StreamAnimationCore, layer_id: usize, frame: Duration) -> impl 'a+Future<Output=KeyFrameCore> {
        async move {
            // Request the keyframe from the core
            let responses = core.request(vec![StorageCommand::ReadElementsForKeyFrame(layer_id, frame)]).await.unwrap_or_else(|| vec![]);

            // Process the elements
            enum ElementSlot<T> {
                Unresolved(T),
                Resolved(ElementWrapper)
            }
            //let mut unresolved  = HashMap::new();
            let mut start_time  = frame;
            let mut end_time    = frame;

            for response in responses {
                use self::StorageResponse::*;

                match response {
                    KeyFrame(start, end)                => { start_time = start; end_time = end; }
                    Element(element_id, serialized)     => { }

                    _                                   => { }
                }
            }

            // Resolve the elements

            unimplemented!()
        }
    }
}
