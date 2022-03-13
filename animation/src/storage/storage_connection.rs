use super::storage_command::*;
use super::storage_response::*;
use super::storage_keyframe::*;
use super::layer_properties::*;
use crate::traits::*;
use crate::editor::element_wrapper::*;
use crate::serializer::*;

use flo_stream::*;
use futures::prelude::*;
use futures::stream::{BoxStream};

use std::ops::{Range};
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Represents a storage command publisher and its connected response stream
///
pub struct StorageConnection {
    storage_requests:   Publisher<Vec<StorageCommand>>, 
    storage_responses:  BoxStream<'static, Vec<StorageResponse>>
}

impl StorageConnection {
    ///
    /// Creates a new storage connection
    ///
    pub fn new(storage_requests: Publisher<Vec<StorageCommand>>, storage_responses: BoxStream<'static, Vec<StorageResponse>>) -> StorageConnection {
        StorageConnection {
            storage_requests,
            storage_responses
        }
    }

    ///
    /// Sends a request to the storage layer
    ///
    pub fn request<'a, Commands: 'a+IntoIterator<Item=StorageCommand>>(&'a mut self, request: Commands) -> impl 'a+Future<Output=Option<Vec<StorageResponse>>> {
        async move {
            self.storage_requests.publish(request.into_iter().collect()).await;
            self.storage_responses.next().await
        }
    }

    ///
    /// Sends a single request that produces a single response to the storage layer
    ///
    pub fn request_one<'a>(&'a mut self, request: StorageCommand) -> impl 'a+Future<Output=Option<StorageResponse>> {
        async move {
            self.request(vec![request]).await
                .and_then(|mut result| result.pop())
        }
    }

    ///
    /// Reads the properties for a single layer from this connection
    ///
    pub fn read_layer_properties<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=Option<LayerProperties>> {
        async move {
            // Retrieve the serialized layer properties
            let responses           = self.request(vec![StorageCommand::ReadLayerProperties(layer_id)]).await?;
            let layer_properties    = responses.into_iter()
                .filter_map(|response| match response {
                    StorageResponse::LayerProperties(_, properties) => Some(properties),
                    _                                               => None
                })
                .next()?;

            // Deserialize to generate the results
            LayerProperties::deserialize(&mut layer_properties.chars())
        }
    }

    ///
    /// Reads the IDs of the layers making up the animation
    ///
    pub fn read_all_layer_properties<'a>(&'a mut self) -> impl 'a+Future<Output=Vec<(u64, LayerProperties)>> {
        async move {
            let responses = self.request(vec![StorageCommand::ReadLayers]).await.unwrap_or_else(|| vec![]);

            responses.into_iter()
                .flat_map(|response| {
                    match response {
                        StorageResponse::LayerProperties(id, props)     => Some((id, LayerProperties::deserialize(&mut props.chars())?)),
                        _                                               => None
                    }
                })
                .collect()
        }
    }

    ///
    /// Reads the IDs of the layers making up the animation
    ///
    pub fn read_layer_ids<'a>(&'a mut self) -> impl 'a+Future<Output=Vec<u64>> {
        async move {
            let responses = self.request(vec![StorageCommand::ReadLayers]).await.unwrap_or_else(|| vec![]);

            responses.into_iter()
                .flat_map(|response| {
                    match response {
                        StorageResponse::LayerProperties(id, _props)    => Some(id),
                        _                                               => None
                    }
                })
                .collect()
        }
    }

    ///
    /// Reads all of the keyframes for a layer within a particular time range
    ///
    pub fn read_keyframes_for_layer<'a>(&'a mut self, layer_id: u64, range: Range<Duration>) -> impl 'a+Future<Output=Option<Vec<Range<Duration>>>> {
        async move {
            // Request the keyframes from the core
            let responses = self.request(vec![StorageCommand::ReadKeyFrames(layer_id, range)]).await.unwrap_or_else(|| vec![]);

            if responses.len() == 1 && responses[0] == StorageResponse::NotFound {
                // Layer doesn't exist
                None
            } else {
                // Convert the responses into keyframe times
                Some(responses.into_iter()
                    .flat_map(|response| {
                        match response {
                            StorageResponse::KeyFrame(from, until)  => Some(from..until),
                            _                                       => None
                        }
                    })
                    .collect())
            }
        }
    }

    ///
    /// Reads all of the elements in a particular keyframe
    ///
    pub fn read_keyframe<'a>(&'a mut self, layer_id: u64, keyframe: Duration) -> impl 'a+Future<Output=Option<StorageKeyFrame>> {
        async move {
            // Request the keyframe from the core
            let responses = self.request(vec![StorageCommand::ReadElementsForKeyFrame(layer_id, keyframe)]).await.unwrap_or_else(|| vec![]);

            // Deserialize the elements for the keyframe
            let mut element_ids = vec![];
            let mut elements    = HashMap::new();
            let mut start_time  = keyframe;
            let mut end_time    = keyframe;

            for response in responses {
                use self::StorageResponse::*;

                match response {
                    NotFound                            => { return None; }
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

            for element_id in element_ids.iter() {
                if let Some(resolver) = elements.remove(element_id) {
                    // Element needs to be resolved
                    if let Some(resolver) = resolver {
                        // Attempt to resolve this element using the others that are attached to this keyframe
                        let resolved_element = resolver.resolve(&mut |element_id| {
                            resolve_element(&mut elements, &mut resolved, element_id)
                                .map(|resolved| resolved.element.clone())
                        });

                        // Store the resolved element
                        if let Some(resolved_element) = resolved_element {
                            if let Vector::Error = &resolved_element.element {
                                warn!("Element {:?} failed to deserialize", *element_id);
                            }
                            resolved.insert(*element_id, resolved_element);
                        } else {
                            warn!("Element {:?} was referenced for this frame but resolved to no element", *element_id);
                        }
                    } else {
                        // Element cannot be resolved
                        warn!("Element {:?} was referenced for this frame but cannot be resolved", *element_id);
                        resolved.insert(*element_id, ElementWrapper::error());
                    }
                } else {
                    // Already resolved this element so there's nothing more to do
                }
            }

            Some(StorageKeyFrame {
                start_time:     start_time,
                end_time:       end_time,
                elements:       resolved,
                element_ids:    element_ids
            })
        }
    }
}

///
/// Resolves an element from a partially resolved list of elements
///
fn resolve_element<'a, Resolver>(unresolved: &mut HashMap<ElementId, Option<Resolver>>, resolved: &'a mut HashMap<ElementId, ElementWrapper>, element_id: ElementId) -> Option<ElementWrapper> 
where Resolver: ResolveElements<ElementWrapper> {
    if let Some(resolved_element) = resolved.get(&element_id) {
        // Already resolved
        Some(resolved_element.clone())
    } else if let Some(Some(unresolved_element)) = unresolved.remove(&element_id) {
        // Exists but is not yet resolved (need to resolve recursively)
        let resolved_element = unresolved_element.resolve(&mut |element_id| { 
            resolve_element(unresolved, resolved, element_id)
                .map(|resolved| resolved.element.clone())
            });

        if let Some(resolved_element) = resolved_element {
            // Resolved the element: add to the resolved list, and return a reference
            resolved.insert(element_id, resolved_element);
            resolved.get(&element_id).cloned()
        } else {
            // Failed to resolve this element
            resolved.insert(element_id, ElementWrapper::error());
            resolved.get(&element_id).cloned()
        }
    } else {
        // Not found
        None
    }
}
