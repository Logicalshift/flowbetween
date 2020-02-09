use super::core::*;
use super::element_wrapper::*;
use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::super::traits::*;
use super::super::super::serializer::*;

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

            // Deserialize the elements for the keyframe
            enum ElementSlot<T> {
                Unresolved(T),
                Resolved(ElementWrapper)
            }
            let mut element_ids = vec![];
            let mut elements    = HashMap::new();
            let mut start_time  = frame;
            let mut end_time    = frame;

            for response in responses {
                use self::StorageResponse::*;

                match response {
                    KeyFrame(start, end)                => { start_time = start; end_time = end; }
                    Element(element_id, serialized)     => {
                        // Add the element to the list we know about for this keyframe
                        let element_id  = ElementId::Assigned(element_id);
                        let element     = ElementWrapper::deserialize(element_id, &mut serialized.chars());

                        elements.insert(element_id, element);
                        element_ids.push(element_id);
                    }

                    _                                   => { }
                }
            }

            // Attempt to resolve the elements (missing elements will be changed to error elements)
            let mut resolved = HashMap::<ElementId, ElementWrapper>::new();

            for element_id in element_ids {
                if let Some(resolver) = elements.remove(&element_id) {
                    // Element needs to be resolved
                    if let Some(resolver) = resolver {
                        // Attempt to resolve this element using the others that are attached to this keyframe
                        let resolved_element = resolver.resolve(&mut |element_id| {
                            if let Some(element) = resolved.get(&element_id) {
                                // Already resolved
                                Some(element.element.clone())
                            } else if let Some(unresolved) = elements.remove(&element_id) {
                                // Exists but is not yet resolved (need to resolve recursively)
                                unimplemented!()
                            } else {
                                // Not found
                                unimplemented!()
                            }
                        });
                    } else {
                        // Element cannot be resolved
                        resolved.insert(element_id, ElementWrapper::error());
                    }
                } else {
                    // Already resolved this element
                }
            }

            unimplemented!()
        }
    }
}
