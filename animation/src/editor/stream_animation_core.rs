use super::keyframe_core::*;
use crate::storage::storage_api::*;
use crate::storage::file_properties::*;
use crate::traits::*;

use ::desync::*;
use flo_stream::*;

use futures::future;
use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Performs an asynchronous request on a storage layer for this animation
///
pub (super) fn request_core_async(core: &Arc<Desync<StreamAnimationCore>>, request: Vec<StorageCommand>) -> impl Future<Output=Option<Vec<StorageResponse>>> {
    core.future_desync(move |core| {
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
pub (super) fn request_core_sync(core: Arc<Desync<StreamAnimationCore>>, request: Vec<StorageCommand>) -> Option<Vec<StorageResponse>> {
    // Queue the request
    let result = core.future_desync(|core| {
        async move {
            core.storage_requests.publish(request).await;
            core.storage_responses.next().await
        }.boxed()
    }).sync().unwrap_or(None);

    // Return the result of the request
    result
}

pub (super) struct StreamAnimationCore {
    /// Stream where responses to the storage requests are sent
    pub (super) storage_responses: BoxStream<'static, Vec<StorageResponse>>,

    /// Publisher where we can send requests for storage actions
    pub (super) storage_requests: Publisher<Vec<StorageCommand>>,

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
    pub (super) path_brush_props: Option<Arc<BrushPropertiesElement>>
}

impl StreamAnimationCore {
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

                Element(elements, Group(group_id, group_type)) =>
                    Element(elements.clone(), Group(self.assign_element_id(*group_id).await, *group_type)),

                other => other.clone()
            }
        }
    }

    ///
    /// Performs a set of edits on the core
    ///
    pub fn perform_edits<'a>(&'a mut self, edits: Arc<Vec<AnimationEdit>>) -> impl 'a+Future<Output=()> {
        async move {
            // Assign IDs to the edits
            let mut mapped_edits    = Vec::with_capacity(edits.len());
            for edit in edits.iter() {
                mapped_edits.push(self.assign_element_id_to_edit_log(edit).await);
            }
            let edits               = mapped_edits;

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
                    Layer(layer_id, layer_edit)             => { self.layer_edit(*layer_id, layer_edit).await; }
                    Element(element_ids, element_edit)      => { self.element_edit(element_ids, element_edit).await; }
                    Motion(motion_id, motion_edit)          => { self.motion_edit(*motion_id, motion_edit).await; }
                    SetSize(width, height)                  => { self.set_size(*width, *height).await }
                    AddNewLayer(layer_id)                   => { self.add_new_layer(*layer_id).await; }
                    RemoveLayer(layer_id)                   => { self.remove_layer(*layer_id).await; }
                }
            }
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
    /// Sets the size of the animation
    ///
    pub fn set_size<'a>(&'a mut self, width: f64, height: f64) -> impl 'a+Future<Output=()> {
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
        }
    }

    ///
    /// Adds a key frame to a layer
    ///
    pub fn add_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=()> { 
        async move {
            self.cached_keyframe = None;
            self.cached_layers.remove(&layer_id);
            self.request_one(StorageCommand::AddKeyFrame(layer_id, when)).await;
        } 
    }

    ///
    /// Removes a key frame from a layer
    ///
    pub fn remove_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=()> { 
        async move {
            self.cached_keyframe = None;
            self.cached_layers.remove(&layer_id);
            self.request_one(StorageCommand::DeleteKeyFrame(layer_id, when)).await;
        } 
    }
}
