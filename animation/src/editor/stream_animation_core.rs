use super::keyframe_core::*;
use super::element_wrapper::*;
use crate::undo::*;
use crate::traits::*;
use crate::storage::*;
use crate::storage::file_properties::*;

use ::desync::*;
use flo_stream::*;

use smallvec::*;
use futures::future;
use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap, HashSet};

///
/// Performs an asynchronous request on a storage layer for this animation
///
pub (super) fn request_core_async(core: &Arc<Desync<StreamAnimationCore>>, request: Vec<StorageCommand>) -> impl Future<Output=Option<Vec<StorageResponse>>> {
    core.future_desync(move |core| {
        async move {
            core.storage_connection.request(request).await
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
pub (super) fn request_core_sync(core: Arc<Desync<StreamAnimationCore>>, request: Vec<StorageCommand>) -> Option<Vec<StorageResponse>> {
    // Queue the request
    let result = core.future_desync(|core| {
        async move {
            core.storage_connection.request(request).await
        }.boxed()
    }).sync().unwrap_or(None);

    // Return the result of the request
    result
}

pub (super) struct StreamAnimationCore {
    /// The connection to the storage sub-system
    pub (super) storage_connection: StorageConnection,

    /// The next element ID to assign (None if we haven't retrieved the element ID yet)
    pub (super) next_element_id: Option<i64>,

    /// Cached loaded layers
    pub (super) cached_layers: HashMap<u64, Arc<KeyFrameCore>>,

    /// The keyframe that is currently being edited, if there is one
    pub (super) cached_keyframe: Option<Arc<Desync<KeyFrameCore>>>,

    /// The brush definition to attach to brush strokes
    pub (super) brush_defn: Option<ElementId>,

    /// The properties to attach to brush strokes
    pub (super) brush_props: Option<ElementId>,

    /// The element that should be used as the brush definition for the current path (unassigned if there is none)
    pub (super) path_brush_defn: Option<Arc<BrushDefinitionElement>>,

    /// The element that should be used as the properties for the current path (unassigned if there is none)
    pub (super) path_brush_props: Option<Arc<BrushPropertiesElement>>,

    /// Channels for sending retired editing instructions
    pub (super) retired_edit_senders: Vec<Publisher<RetiredEdit>>,
}

impl StreamAnimationCore {
    ///
    /// Sends a request to the storage layer
    ///
    pub fn request<'a, Commands: 'a+IntoIterator<Item=StorageCommand>>(&'a mut self, request: Commands) -> impl 'a+Future<Output=Option<Vec<StorageResponse>>> {
        self.storage_connection.request(request)
    }

    ///
    /// Sends a single request that produces a single response to the storage layer
    ///
    pub fn request_one<'a>(&'a mut self, request: StorageCommand) -> impl 'a+Future<Output=Option<StorageResponse>> {
        self.storage_connection.request_one(request)
    }

    ///
    /// Returns a mutable copy of the element ID that will be assigned by the next call to next_element_id
    ///
    fn next_element_id<'a>(&'a mut self) -> impl 'a+Future<Output=&'a mut i64> {
        async move {
            if self.next_element_id.is_some() {
                // Use the existing next element ID
                self.next_element_id.as_mut().unwrap()
            } else {
                // Load the next element ID from storage
                let next_id = match self.request_one(StorageCommand::ReadHighestUnusedElementId).await {
                    Some(StorageResponse::HighestUnusedElementId(next_id))  => next_id,
                    _                                                       => { panic!("No next element ID is available"); }
                };

                self.next_element_id = Some(next_id+1);
                self.next_element_id.as_mut().unwrap()
            }
        }
    }

    ///
    /// Ensures that an element ID has an assigned value
    ///
    pub fn assign_element_id<'a>(&'a mut self, element_id: ElementId) -> impl 'a+Future<Output=ElementId> {
        async move {
            if let ElementId::Assigned(_) = element_id {
                // Nothing to do if the element ID is already assigned
                element_id
            } else {
                let next_element_id = if let Some(element_id) = self.next_element_id.as_mut() {
                    // Add one to the existing ID
                    let next_id = *element_id;
                    *element_id += 1;

                    next_id
                } else {
                    // Fetch the element ID from the storage
                    let next_id = match self.request_one(StorageCommand::ReadHighestUnusedElementId).await {
                        Some(StorageResponse::HighestUnusedElementId(next_id))  => next_id,
                        _                                                       => { return ElementId::Unassigned; }
                    };

                    // Next ID to return is the one after this
                    self.next_element_id = Some(next_id+1);

                    // Use this ID
                    next_id
                };

                // Assign the element to the next available ID
                ElementId::Assigned(next_element_id)
            }
        }
    }

    ///
    /// Updates any edit log entries so they don't use an unassigned element ID
    /// 
    /// (We want to do this before writing to the log so that IDs will be consistent over time)
    ///
    pub fn assign_element_id_to_edit_log<'a>(&'a mut self, edit: &'a AnimationEdit) -> impl 'a+Future<Output=AnimationEdit> {
        async move {
            use self::AnimationEdit::*;
            use self::LayerEdit::*;
            use self::PaintEdit::*;
            use self::ElementEdit::*;

            match edit {
                Layer(layer_id, Paint(when, BrushProperties(element, props))) =>
                    Layer(*layer_id, Paint(*when, BrushProperties(self.assign_element_id(*element).await, props.clone()))),

                Layer(layer_id, Paint(when, SelectBrush(element, defn, drawing_style))) =>
                    Layer(*layer_id, Paint(*when, SelectBrush(self.assign_element_id(*element).await, defn.clone(), *drawing_style))),

                Layer(layer_id, Paint(when, BrushStroke(element, points))) =>
                    Layer(*layer_id, Paint(*when, BrushStroke(self.assign_element_id(*element).await, points.clone()))),

                Layer(layer_id, Paint(when, CreateShape(element, width, shape))) =>
                    Layer(*layer_id, Paint(*when, CreateShape(self.assign_element_id(*element).await, *width, shape.clone()))),

                Layer(layer_id, Paint(when, Fill(element, point, options))) =>
                    Layer(*layer_id, Paint(*when, Fill(self.assign_element_id(*element).await, point.clone(), options.clone()))),

                Layer(layer_id, Path(when, PathEdit::CreatePath(element, points))) =>
                    Layer(*layer_id, Path(*when, PathEdit::CreatePath(self.assign_element_id(*element).await, points.clone()))),

                Layer(layer_id, Path(when, PathEdit::SelectBrush(element, definition, style))) =>
                    Layer(*layer_id, Path(*when, PathEdit::SelectBrush(self.assign_element_id(*element).await, definition.clone(), *style))),

                Layer(layer_id, Path(when, PathEdit::BrushProperties(element, properties))) =>
                    Layer(*layer_id, Path(*when, PathEdit::BrushProperties(self.assign_element_id(*element).await, properties.clone()))),

                Layer(layer_id, CreateAnimation(when, element, description)) =>
                    Layer(*layer_id, CreateAnimation(*when, self.assign_element_id(*element).await, description.clone())),

                Layer(layer_id, CreateElement(when, element, vector)) =>
                    Layer(*layer_id, CreateElement(*when, self.assign_element_id(*element).await, vector.clone())),

                Element(elements, Group(group_id, group_type)) =>
                    Element(elements.clone(), Group(self.assign_element_id(*group_id).await, *group_type)),

                other => other.clone()
            }
        }
    }

    ///
    /// Performs a set of edits on the core
    ///
    pub fn perform_edits<'a>(&'a mut self, edits: Arc<Vec<AnimationEdit>>) -> impl 'a+Future<Output=RetiredEdit> {
        async move {
            // If the edits contain element IDs that have not been used before, ensure that they're not returned by assign_element_id()
            let mut max_element_id = None;

            for edit_element_id in edits.iter().flat_map(|edit| edit.used_element_ids()).filter_map(|element_id| element_id.id()) {
                if let Some(max_element_id) = &mut max_element_id {
                    *max_element_id = i64::max(*max_element_id, edit_element_id);
                } else {
                    max_element_id = Some(edit_element_id);
                }
            }

            if let Some(max_element_id) = max_element_id {
                let next_assign_id = self.next_element_id().await;
                if max_element_id >= *next_assign_id {
                    *next_assign_id = max_element_id + 1;
                }
            }

            // Assign IDs to the edits
            let mut mapped_edits    = Vec::with_capacity(edits.len());
            for edit in edits.iter() {
                mapped_edits.push(self.assign_element_id_to_edit_log(edit).await);
            }
            let edits               = mapped_edits;
            let mut reversed_edits  = ReversedEdits::new();

            // Send the edits to the edit log by serializing them
            let edit_log = edits.iter()
                .map(|edit| {
                    let mut serialized = String::new();
                    edit.serialize(&mut serialized);
                    serialized
                })
                .map(|edit| StorageCommand::WriteEdit(edit))
                .collect::<Vec<_>>();

            self.request(edit_log).await;

            // Process the edits in the order that they arrive
            for edit in edits.iter() {
                use self::AnimationEdit::*;

                // Edit the elements
                match edit {
                    Layer(layer_id, layer_edit)             => { reversed_edits.add_to_start(self.layer_edit(*layer_id, layer_edit).await); }
                    Element(element_ids, element_edit)      => { reversed_edits.add_to_start(self.element_edit(element_ids, element_edit).await); }
                    Motion(motion_id, motion_edit)          => { reversed_edits.add_to_start(self.motion_edit(*motion_id, motion_edit).await); }
                    SetSize(width, height)                  => { reversed_edits.add_to_start(self.set_size(*width, *height).await) }
                    SetFrameLength(length)                  => { reversed_edits.add_to_start(self.set_frame_length(*length).await) }
                    SetLength(length)                       => { reversed_edits.add_to_start(self.set_length(*length).await) }
                    AddNewLayer(layer_id)                   => { reversed_edits.add_to_start(self.add_new_layer(*layer_id).await); }
                    RemoveLayer(layer_id)                   => { reversed_edits.add_to_start(self.remove_layer(*layer_id).await); }
                }
            }

            // TODO: generate the 'reverse' edits
            RetiredEdit::new(Arc::new(edits), reversed_edits.into())
        }
    }

    ///
    /// Loads the keyframe containing the specified moment
    ///
    pub fn load_keyframe<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a + Future<Output=Option<Arc<KeyFrameCore>>> {
        async move {
            // We use a cached keyframe if possible. This saves us having to recalculate the animation layer and reload the elements
            // from storage, but we need to be careful when editing the keyframe as the cached version will become out of date.

            // Try to fetch an existing keyframe if possible
            let existing_keyframe = if let Some(keyframe) = self.cached_layers.get(&layer_id) {
                if keyframe.start <= when && keyframe.end > when {
                    Some(Arc::clone(keyframe))
                } else {
                    // We free the cached keyframe before loading a new one if it's unused
                    self.cached_layers.remove(&layer_id);
                    None
                }
            } else {
                None
            };

            // Load a new keyframe if no existing keyframe could be found
            if existing_keyframe.is_none() {
                let new_keyframe = KeyFrameCore::from_keyframe(self, layer_id, when).await
                    .map(|frame| Arc::new(frame));

                if let Some(new_keyframe) = new_keyframe {
                    self.cached_layers.insert(layer_id, Arc::clone(&new_keyframe));
                    Some(new_keyframe)
                } else {
                    None
                }
            } else {
                existing_keyframe
            }
        }
    }

    ///
    /// Updates the cached keyframe to be at the specific time/layer if it's not already
    ///
    pub fn edit_keyframe<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a + Future<Output=Option<Arc<Desync<KeyFrameCore>>>> {
        async move {
            // Force the next fetch of this layer to update the frame
            // TODO: if the keyframe is cached again and then edited, the cache will get out of sync: we need a way to either share the 
            // keyframe with the cache or invalidate it when it's edited (this works for now but is very fragile and might be hard to debug 
            // for anyone who hasn't read or remembered this comment)
            self.cached_layers.remove(&layer_id);

            // Return the cached keyframe if it matches the layer and time
            if let Some(keyframe) = self.cached_keyframe.as_ref() {
                let (frame_layer_id, start, end) = keyframe.future_sync(|keyframe| future::ready((keyframe.layer_id, keyframe.start, keyframe.end)).boxed()).await.unwrap();

                if frame_layer_id == layer_id && start <= when && (end > when || start == when) {
                    return Some(Arc::clone(keyframe));
                }
            }

            // Update the cached keyframe if it doesn't
            self.cached_keyframe = self.load_keyframe(layer_id, when).await
                .map(|keyframe| Arc::new(Desync::new((*keyframe).clone())));

            // This will re-cache keyframe for this layer, so remove it again
            // TODO: would be way better for have the cached keyframes be desync if possible
            self.cached_layers.remove(&layer_id);

            // This is the result of the operation
            self.cached_keyframe.clone()
        }
    }

    ///
    /// Attempts to load the keyframe for the specified element for editing 
    ///
    pub fn edit_keyframe_for_element<'a>(&'a mut self, element_id: i64) -> impl 'a + Future<Output=Option<Arc<Desync<KeyFrameCore>>>> {
        async move {
            // Fetch the keyframe that the root element is in
            let keyframe_response = self.request_one(StorageCommand::ReadElementAttachments(element_id)).await;

            if let Some(StorageResponse::ElementAttachments(_elem, mut keyframes)) = keyframe_response  {
                if let Some((layer_id, keyframe_time)) = keyframes.pop() {
                    // Need to retrieve the keyframe (some elements depend on others, so we need the whole keyframe)
                    // Most of the time we'll be editing a single frame so this won't be too expensive
                    self.cached_layers.remove(&layer_id);
                    self.edit_keyframe(layer_id, keyframe_time).await
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    ///
    /// Finds the wrappers for the elements with the specified IDs
    ///
    pub fn wrappers_for_elements<'a>(&'a mut self, elements: impl 'a + Send+Iterator<Item=i64>) -> impl 'a + Future<Output=HashMap<i64, ElementWrapper>> {
        async move {
            let mut remaining           = elements.collect::<SmallVec<[_; 4]>>();
            let mut wrappers            = HashMap::new();

            // While there's another element...
            while let Some(next_element) = remaining.last() {
                // Fetch the keyframe corresponding to the next element
                let next_element    = *next_element;
                let keyframe        = self.edit_keyframe_for_element(next_element).await;

                if let Some(keyframe) = keyframe {
                    // Retrieve as many elements as possible from the keyframe and update the wrappers list and the remaining list of elements in other keyframes
                    (remaining, wrappers) = keyframe.future_sync(move |keyframe| {
                        async move {
                            debug_assert!(keyframe.elements.contains_key(&ElementId::Assigned(next_element)), "Element missing from keyframe");

                            // The elements that can't be found in this frame
                            let mut not_in_frame = smallvec![];

                            // Try to resolve all the elements we can. There should always be at least one (the one we used to look up the keyframe)
                            for element_id in remaining {
                                if let Some(wrapper) = keyframe.elements.get(&ElementId::Assigned(element_id)) {
                                    wrappers.insert(element_id, wrapper.clone());
                                } else {
                                    not_in_frame.push(element_id);
                                }
                            }

                            (not_in_frame, wrappers)
                        }.boxed()
                    }).await.unwrap();
                } else {
                    // This element has no keyframe, so we can't look up the wrapper
                    remaining.pop();
                }
            }

            wrappers
        }
    }

    ///
    /// Sets the size of the animation
    ///
    pub fn set_size<'a>(&'a mut self, width: f64, height: f64) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Get the current animation properties
            let properties      = self.request_one(StorageCommand::ReadAnimationProperties).await;
            let properties      = if let Some(StorageResponse::AnimationProperties(properties)) = properties {
                FileProperties::deserialize(&mut properties.chars())
            } else {
                None
            };
            let mut properties  = properties.unwrap_or_else(|| FileProperties::default());

            self.cached_layers.clear();

            // Update the size
            properties.size     = (width, height);

            // Send the new file size to the storage
            let mut new_properties = String::new();
            properties.serialize(&mut new_properties);
            self.request_one(StorageCommand::WriteAnimationProperties(new_properties)).await;

            ReversedEdits::unimplemented()
        }
    }

    ///
    /// Sets the length of a frame in the animation
    ///
    pub fn set_frame_length<'a>(&'a mut self, frame_length: Duration) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Get the current animation properties
            let properties          = self.request_one(StorageCommand::ReadAnimationProperties).await;
            let properties          = if let Some(StorageResponse::AnimationProperties(properties)) = properties {
                FileProperties::deserialize(&mut properties.chars())
            } else {
                None
            };
            let mut properties      = properties.unwrap_or_else(|| FileProperties::default());

            self.cached_layers.clear();

            // Update the frame length
            properties.frame_length = frame_length;

            // Send the new file size to the storage
            let mut new_properties = String::new();
            properties.serialize(&mut new_properties);
            self.request_one(StorageCommand::WriteAnimationProperties(new_properties)).await;

            ReversedEdits::unimplemented()
        }
    }

    ///
    /// Sets the length of the animation as a whole
    ///
    pub fn set_length<'a>(&'a mut self, length: Duration) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Get the current animation properties
            let properties          = self.request_one(StorageCommand::ReadAnimationProperties).await;
            let properties          = if let Some(StorageResponse::AnimationProperties(properties)) = properties {
                FileProperties::deserialize(&mut properties.chars())
            } else {
                None
            };
            let mut properties      = properties.unwrap_or_else(|| FileProperties::default());

            self.cached_layers.clear();

            // Update the length of the animation
            properties.duration     = length;

            // Send the new file size to the storage
            let mut new_properties = String::new();
            properties.serialize(&mut new_properties);
            self.request_one(StorageCommand::WriteAnimationProperties(new_properties)).await;

            ReversedEdits::unimplemented()
        }
    }

    ///
    /// Adds a key frame to a layer
    ///
    pub fn add_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=ReversedEdits> { 
        async move {
            self.cached_keyframe = None;
            self.cached_layers.remove(&layer_id);
            self.request_one(StorageCommand::AddKeyFrame(layer_id, when)).await;

            ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::RemoveKeyFrame(when)))
        } 
    }

    ///
    /// Removes a key frame from a layer
    ///
    pub fn remove_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=ReversedEdits> { 
        async move {
            // Create the undo action
            let reverse_action = ReversedEdits::with_recreated_keyframe(layer_id, when, &mut HashSet::new(), &mut self.storage_connection).await;

            // Remove from the cache
            self.cached_keyframe = None;
            self.cached_layers.remove(&layer_id);

            // Request that the storage layer deletes the keyframe
            self.request_one(StorageCommand::DeleteKeyFrame(layer_id, when)).await;

            reverse_action
        } 
    }
}
