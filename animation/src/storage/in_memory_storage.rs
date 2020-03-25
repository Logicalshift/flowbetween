use super::storage_api::*;

use ::desync::*;

use futures::prelude::*;
use futures::future;

use std::i64;
use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Represents a key frame
///
struct InMemoryKeyFrameStorage {
    /// The time when this frame appears
    when: Duration,

    /// The IDs of the elements attached to this keyframe
    attached_elements: HashMap<i64, Duration>
}

///
/// Represents a layer cache item
///
struct InMemoryLayerCache {
    /// The key for this cache item
    key: String,

    /// When this item is stored
    when: Duration,

    /// The value of this cache item
    cache_value: String
}

///
/// Representation of a layer in memory
///
struct InMemoryLayerStorage {
    /// The properties for this layer
    properties: String,

    /// The keyframes of this layer
    keyframes: Vec<InMemoryKeyFrameStorage>,

    /// The cached items for this layer
    cache: Vec<InMemoryLayerCache>
}

///
/// Indicates where an element is attached
///
struct ElementAttachment {
    layer_id:       u64,
    keyframe_time:  Duration
}

///
/// Representation of an animation in-memory
///
struct InMemoryStorageCore {
    /// The properties for the animation
    animation_properties: Option<String>,

    /// The edit log
    edit_log: Vec<String>,

    /// The definitions for each element
    elements: HashMap<i64, String>,

    /// The keyframes that an element is attached to
    element_attachments: HashMap<i64, Vec<ElementAttachment>>,

    /// The layers
    layers: HashMap<u64, InMemoryLayerStorage>
}

///
/// Provides an implementation of the storage API that stores its data in memory
///
pub struct InMemoryStorage {
    /// Where the data is stored for this object 
    storage: Arc<Desync<InMemoryStorageCore>>
}

impl InMemoryStorage {
    ///
    /// Creates a new in-memory storage for an animation
    ///
    pub fn new() -> InMemoryStorage {
        // Create the core
        let core = InMemoryStorageCore {
            animation_properties:   None,
            edit_log:               vec![],
            elements:               HashMap::new(),
            layers:                 HashMap::new(),
            element_attachments:    HashMap::new()
        };

        // And the storage
        InMemoryStorage {
            storage: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Returns the responses for a stream of commands
    ///
    pub fn get_responses<CommandStream: 'static+Send+Unpin+Stream<Item=Vec<StorageCommand>>>(&self, commands: CommandStream) -> impl Send+Unpin+Stream<Item=Vec<StorageResponse>> {
        pipe(Arc::clone(&self.storage), commands, |storage, commands| {
            future::ready(storage.run_commands(commands)).boxed()
        })
    }
}

impl InMemoryStorageCore {
    ///
    /// Removes all the attached elements from the specified keyframe
    ///
    fn detach_elements_from_keyframe(layer_id: u64, keyframe: &mut InMemoryKeyFrameStorage, element_attachments: &mut HashMap<i64, Vec<ElementAttachment>>) {
        // The attached elements are identified by layer ID and the keyframe time
        let keyframe_time = keyframe.when;

        // Remove all the elements from the element_attachments list
        for (elem_id, _) in keyframe.attached_elements.iter() {
            if let Some(attachments) = element_attachments.get_mut(elem_id) {
                attachments.retain(|attachment| attachment.layer_id != layer_id || attachment.keyframe_time != keyframe_time);
            }
        }

        // Clear out the elements from the keyframe itself
        keyframe.attached_elements.clear();
    }

    ///
    /// Removes all of the references to an element in a particular layer
    ///
    fn detach_all_elements_in_layer(&mut self, layer_id: u64) {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            for keyframe in layer.keyframes.iter_mut() {
                Self::detach_elements_from_keyframe(layer_id, keyframe, &mut self.element_attachments);
            }
        }
    }

    ///
    /// Removes an element from all of its attachments
    ///
    fn detach_element(&mut self, element_id: i64) -> bool {
        // Remove the attachments from the storage
        if let Some(attachments) = self.element_attachments.remove(&element_id) {
            // Remove the elements from each of the keyframes it's attached to
            for attachment in attachments.into_iter() {
                // Fetch the layer
                if let Some(layer) = self.layers.get_mut(&attachment.layer_id) {
                    // Search for the keyframe
                    if let Ok(keyframe_index) = layer.keyframes.binary_search_by(|frame| frame.when.cmp(&attachment.keyframe_time)) {
                        // Remove the element from the keyframe
                        layer.keyframes[keyframe_index].attached_elements.remove(&element_id);
                    }
                }
            }

            true
        } else {
            // Element not found
            false
        }
    }

    ///
    /// Runs a series of storage commands on this store
    ///
    pub fn run_commands(&mut self, commands: Vec<StorageCommand>) -> Vec<StorageResponse> {
        let mut response = vec![];

        for command in commands.into_iter() {
            use self::StorageCommand::*;

            match command {
                WriteAnimationProperties(props)                     => { 
                    self.animation_properties = Some(props); 
                    response.push(StorageResponse::Updated); 
                }

                ReadAnimationProperties                             => { 
                    response.push(self.animation_properties.as_ref()
                        .map(|props| StorageResponse::AnimationProperties(props.clone()))
                        .unwrap_or(StorageResponse::NotFound)); 
                }

                WriteEdit(edit)                                     => { 
                    self.edit_log.push(edit); 
                    response.push(StorageResponse::Updated); 
                }

                ReadHighestUnusedElementId                          => { 
                    response.push(StorageResponse::HighestUnusedElementId(self.elements.keys().cloned().max().unwrap_or(-1)+1)); 
                }

                ReadEditLogLength                                   => { 
                    response.push(StorageResponse::NumberOfEdits(self.edit_log.len())); 
                }

                ReadEdits(edit_range)                               => { 
                    response.extend(edit_range.into_iter()
                        .map(|index| StorageResponse::Edit(index, self.edit_log[index].clone()))); 
                }

                WriteElement(element_id, value)                     => { 
                    self.elements.insert(element_id, value); 
                    response.push(StorageResponse::Updated);
                }

                ReadElement(element_id)                             => { 
                    response.push(self.elements.get(&element_id)
                        .map(|element| StorageResponse::Element(element_id, element.clone()))
                        .unwrap_or(StorageResponse::NotFound)); 
                }

                AddLayer(layer_id, properties)                      => { 
                    self.layers.insert(layer_id, InMemoryLayerStorage::new(properties)); 
                    response.push(StorageResponse::Updated); 
                }

                DeleteElement(element_id)                           => { 
                    if let Some(_element) = self.elements.remove(&element_id) {
                        self.detach_element(element_id);
                        response.push(StorageResponse::Updated); 
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }
                
                DeleteLayer(layer_id)                               => { 
                    if self.layers.remove(&layer_id).is_some() { 
                        self.detach_all_elements_in_layer(layer_id);
                        response.push(StorageResponse::Updated); 
                    } else { 
                        response.push(StorageResponse::NotFound); 
                    }
                }

                ReadLayers                                          => { 
                    for (layer_id, storage) in self.layers.iter() {
                        response.push(StorageResponse::LayerProperties(*layer_id, storage.properties.clone()));
                    }
                }
                
                WriteLayerProperties(layer_id, properties)          => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.properties = properties;
                        response.push(StorageResponse::Updated);
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadLayerProperties(layer_id)                       => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        response.push(StorageResponse::LayerProperties(layer_id, layer.properties.clone()));
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                AddKeyFrame(layer_id, when)                         => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the location where the keyframe can be added
                        match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            Ok(_)           => {
                                // This keyframe already exists
                                response.push(StorageResponse::NotReplacingExisting)
                            }

                            Err(location)   => {
                                // Need to add a new keyframe
                                let keyframe = InMemoryKeyFrameStorage::new(when);
                                layer.keyframes.insert(location, keyframe);

                                response.push(StorageResponse::Updated);
                            }
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                DeleteKeyFrame(layer_id, when)                      => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the location where the keyframe needs to be removed
                        match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            Ok(location)    => {
                                // Exact match of a keyframe
                                Self::detach_elements_from_keyframe(layer_id, &mut layer.keyframes[location], &mut self.element_attachments);
                                layer.keyframes.remove(location);

                                response.push(StorageResponse::Updated)
                            }

                            Err(_)          => {
                                // No keyframe at this location
                                response.push(StorageResponse::NotFound);
                            }
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadKeyFrames(layer_id, period)                     => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        // Search for the initial keyframe
                        let initial_keyframe_index = match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&period.start)) {
                            // Period starts at an exact keyframe
                            Ok(location)    => location,

                            // Period covers the keyframe before the specified location if we get a partial match
                            Err(location)   => if location > 0 { location - 1 } else { location }
                        };

                        // Return keyframes until we reach the end of the period
                        let mut keyframe_index  = initial_keyframe_index;
                        let mut num_keyframes   = 0;
                        while keyframe_index < layer.keyframes.len() && layer.keyframes[keyframe_index].when < period.end {
                            // Work out when this keyframe starts and ends
                            let start   = layer.keyframes[keyframe_index].when;
                            let end     = if keyframe_index+1 < layer.keyframes.len() {
                                layer.keyframes[keyframe_index+1].when
                            } else {
                                Duration::from_micros(i64::max_value() as u64)
                            };

                            // Add to the response
                            response.push(StorageResponse::KeyFrame(start, end));
                            num_keyframes += 1;

                            // Move on to the next keyframe
                            keyframe_index += 1;
                        }

                        if num_keyframes == 0 && keyframe_index < layer.keyframes.len() {
                            // If no keyframes were returned but there's a valid keyframe, indicate where the following keyframe is
                            response.push(StorageResponse::NotInAFrame(layer.keyframes[keyframe_index].when));
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                AttachElementToLayer(layer_id, element_id, when)    => {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the keyframe containing this time
                        let keyframe_index = match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            // Period starts at an exact keyframe
                            Ok(location)    => Some(location),

                            // Period covers the keyframe before the specified location if we get a partial match
                            Err(location)   => if location > 0 { Some(location - 1) } else { None }
                        };

                        if let Some(keyframe_index) = keyframe_index {
                            // Attach to this keyframe
                            layer.keyframes[keyframe_index].attached_elements.insert(element_id, when);

                            self.element_attachments.entry(element_id)
                                .or_insert_with(|| vec![])
                                .push(ElementAttachment {
                                    layer_id:       layer_id, 
                                    keyframe_time:  layer.keyframes[keyframe_index].when
                                });
                        } else {
                            // Keyframe not found
                            response.push(StorageResponse::NotFound);
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                DetachElementFromLayer(element_id)                  => { 
                    if self.detach_element(element_id) {
                        response.push(StorageResponse::Updated);
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadElementAttachments(element_id)                  => {
                    if let Some(attachments) = self.element_attachments.get(&element_id) {
                        // Attachments found for this element
                        let attachments = attachments.iter()
                            .map(|attachment| (attachment.layer_id, attachment.keyframe_time))
                            .collect();

                        response.push(StorageResponse::ElementAttachments(element_id, attachments));
                    } else if !self.elements.contains_key(&element_id) {
                        // Element not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadElementsForKeyFrame(layer_id, when)             => { 
                    if let Some(layer) = self.layers.get(&layer_id) {
                        // Search for the keyframe
                        let keyframe_index = match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            Ok(index)   => Some(index),
                            Err(index)  => if index > 0 { Some(index-1) } else { None }
                        };

                        if let Some(keyframe_index) = keyframe_index {
                            // Found the keyframe: fetch the element IDs and definitions that are attached to it
                            let element_ids     = layer.keyframes[keyframe_index].attached_elements.iter().map(|(element_id, _when)| element_id);
                            let element_defns   = element_ids.map(|id| self.elements.get(&id).map(move |defn| (id, defn.clone()))).flatten();

                            response.extend(element_defns
                                .map(|(element_id, defn)| StorageResponse::Element(*element_id, defn.clone())));
                        } else {
                            // Keyframe not present
                            response.push(StorageResponse::NotFound);
                        }
                    } else {
                        // Layer not present
                        response.push(StorageResponse::NotFound);
                    }
                }

                WriteLayerCache(layer_id, when, key, cache_value)   => {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for this cache item
                        match layer.cache.binary_search_by(|cache_item| cache_item.key.cmp(&key).then(cache_item.when.cmp(&when))) {
                            Ok(index)   => layer.cache[index].cache_value = cache_value,
                            Err(index)  => layer.cache.insert(index, InMemoryLayerCache { key, when, cache_value })
                        }

                        // Indicate we added the cache value
                        response.push(StorageResponse::Updated);
                    } else {
                        // Layer not present
                        response.push(StorageResponse::NotFound);
                    }
                }

                DeleteLayerCache(layer_id, when, key)                 => {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for this cache item
                        match layer.cache.binary_search_by(|cache_item| cache_item.key.cmp(&key).then(cache_item.when.cmp(&when))) {
                            Ok(index)   => {
                                layer.cache.remove(index);
                                response.push(StorageResponse::Updated)
                            },
                            Err(_index) => response.push(StorageResponse::NotFound)
                        }
                    } else {
                        // Layer not present
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadLayerCache(layer_id, when, key)                 => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        // Search for this cache item
                        match layer.cache.binary_search_by(|cache_item| cache_item.key.cmp(&key).then(cache_item.when.cmp(&when))) {
                            Ok(index)   => response.push(StorageResponse::LayerCache(layer.cache[index].cache_value.clone())),
                            Err(_index) => response.push(StorageResponse::NotFound)
                        }
                    } else {
                        // Layer not present
                        response.push(StorageResponse::NotFound);
                    }
                }
            }
        }

        response
    }
}

impl InMemoryLayerStorage {
    ///
    /// Creates a new in-memory layer storage object
    ///
    pub fn new(properties: String) -> InMemoryLayerStorage {
        InMemoryLayerStorage {
            properties: properties,
            keyframes:  vec![],
            cache:      vec![]
        }
    }
}

impl InMemoryKeyFrameStorage {
    ///
    /// Creates a new in-memory keyframe storage object
    ///
    pub fn new(when: Duration) -> InMemoryKeyFrameStorage {
        InMemoryKeyFrameStorage {
            when:               when,
            attached_elements:  HashMap::new()
        }
    }
}
