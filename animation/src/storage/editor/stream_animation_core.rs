use super::keyframe_core::*;
use super::element_wrapper::*;
use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::layer_properties::*;
use super::super::super::traits::*;

use ::desync::*;
use flo_stream::*;

use futures::future;
use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet};

pub (super) struct StreamAnimationCore {
    /// Stream where responses to the storage requests are sent
    pub (super) storage_responses: BoxStream<'static, Vec<StorageResponse>>,

    /// Publisher where we can send requests for storage actions
    pub (super) storage_requests: Publisher<Vec<StorageCommand>>,

    /// The next element ID to assign (None if we haven't retrieved the element ID yet)
    pub (super) next_element_id: Option<i64>,

    /// The keyframe that is currently being edited, if there is one
    pub (super) cached_keyframe: Option<Arc<Desync<KeyFrameCore>>>,

    /// The element that should be used as the brush definition for the current path (unassigned if there is none)
    pub (super) path_brush_defn: Option<Arc<BrushDefinitionElement>>,

    /// The element that should be used as the properties for the current path (unassigned if there is none)
    pub (super) path_brush_props: Option<Arc<BrushPropertiesElement>>
}

impl StreamAnimationCore {
    ///
    /// Sends a request to the storage layer
    ///
    pub fn request<'a>(&'a mut self, request: Vec<StorageCommand>) -> impl 'a+Future<Output=Option<Vec<StorageResponse>>> {
        async move {
            self.storage_requests.publish(request).await;
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

            match edit {
                Layer(layer_id, Paint(when, BrushProperties(element, props))) =>
                    Layer(*layer_id, Paint(*when, BrushProperties(self.assign_element_id(*element).await, props.clone()))),

                Layer(layer_id, Paint(when, SelectBrush(element, defn, drawing_style))) =>
                    Layer(*layer_id, Paint(*when, SelectBrush(self.assign_element_id(*element).await, defn.clone(), *drawing_style))),

                Layer(layer_id, Paint(when, BrushStroke(element, points))) =>
                    Layer(*layer_id, Paint(*when, BrushStroke(self.assign_element_id(*element).await, points.clone()))),

                Layer(layer_id, Path(when, PathEdit::CreatePath(element, points))) =>
                    Layer(*layer_id, Path(*when, PathEdit::CreatePath(self.assign_element_id(*element).await, points.clone()))),

                Layer(layer_id, Path(when, PathEdit::SelectBrush(element, definition, style))) =>
                    Layer(*layer_id, Path(*when, PathEdit::SelectBrush(self.assign_element_id(*element).await, definition.clone(), *style))),

                Layer(layer_id, Path(when, PathEdit::BrushProperties(element, properties))) =>
                    Layer(*layer_id, Path(*when, PathEdit::BrushProperties(self.assign_element_id(*element).await, properties.clone()))),

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
                .collect();

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
    pub fn load_keyframe<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a + Future<Output=Option<KeyFrameCore>> {
        async move {
            KeyFrameCore::from_keyframe(self, layer_id, when).await
        }
    }

    ///
    /// Updates the cached keyframe to be at the specific time/layer if it's not already
    ///
    pub fn edit_keyframe<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a + Future<Output=Option<Arc<Desync<KeyFrameCore>>>> {
        async move {
            // Return the cached keyframe if it matches the layer and time
            if let Some(keyframe) = self.cached_keyframe.as_ref() {
                let (layer_id, start, end) = keyframe.future(|keyframe| future::ready((keyframe.layer_id, keyframe.start, keyframe.end)).boxed()).await.unwrap();

                if layer_id == layer_id && start <= when && end > when {
                    return Some(Arc::clone(keyframe));
                }
            }

            // Update the cached keyframe if it doesn't
            self.cached_keyframe = self.load_keyframe(layer_id, when).await
                .map(|keyframe| Arc::new(Desync::new(keyframe)));
            
            // This is the result of the operation
            self.cached_keyframe.clone()
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

            // Update the size
            properties.size     = (width, height);

            // Send the new file size to the storage
            let mut new_properties = String::new();
            properties.serialize(&mut new_properties);
            self.request_one(StorageCommand::WriteAnimationProperties(new_properties)).await;
        }
    }

    ///
    /// Performs a layer edit on this animation
    ///
    pub fn layer_edit<'a>(&'a mut self, layer_id: u64, layer_edit: &'a LayerEdit) -> impl 'a+Future<Output=()> {
        use self::LayerEdit::*;

        async move {
            match layer_edit {
                Paint(when, paint_edit)     => { self.paint_edit(layer_id, *when, paint_edit).await }
                Path(when, path_edit)       => { self.path_edit(layer_id, *when, path_edit).await }
                AddKeyFrame(when)           => { self.add_key_frame(layer_id, *when).await }
                RemoveKeyFrame(when)        => { self.remove_key_frame(layer_id, *when).await }
                SetName(new_name)           => { self.set_layer_name(layer_id, new_name).await }
                SetOrdering(ordering)       => { self.set_layer_ordering(layer_id, *ordering).await }
            }
        }
    }

    ///
    /// Performs a paint edit on a layer
    ///
    pub fn paint_edit<'a>(&'a mut self, layer_id: u64, when: Duration, edit: &'a PaintEdit) -> impl 'a+Future<Output=()> { 
        async move { 
            use self::PaintEdit::*;

            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return; }
                Some(keyframe)  => keyframe
            };

            let (id, element) = match edit {
                SelectBrush(element_id, defn, style)    => {
                    // Create a brush definition element
                    let defn            = BrushDefinitionElement::new(*element_id, defn.clone(), *style);
                    let element         = Vector::BrushDefinition(defn);
                    let element_id      = element_id.id().unwrap_or(0);

                    (element_id, element)
                }

                BrushProperties(element_id, properties) => {
                    // Create a brush properties element
                    let defn            = BrushPropertiesElement::new(*element_id, properties.clone());
                    let element         = Vector::BrushProperties(defn);
                    let element_id      = element_id.id().unwrap_or(0);

                    (element_id, element)
                }

                BrushStroke(element_id, points)         => {
                    // Create a brush stroke element, using the current brush for the layer
                    let active_brush    = current_keyframe.future(|keyframe| async move { keyframe.get_active_brush() }.boxed()).await.unwrap();
                    let points          = active_brush.brush_points_for_raw_points(points);
                    let brush_element   = BrushElement::new(*element_id, Arc::new(points));
                    let element         = Vector::BrushStroke(brush_element);
                    let element_id      = element_id.id().unwrap_or(0);

                    (element_id, element)
                }
            };

            // Edit the keyframe
            let storage_updates = current_keyframe.future(move |current_keyframe| {
                async move {
                    // Add to a wrapper
                    let wrapper         = ElementWrapper::with_element(element, when);

                    // Append to the current keyframe and return the list of storage commands
                    current_keyframe.add_element_to_end(ElementId::Assigned(id), wrapper)
                }.boxed()
            }).await;

            // Send to the storage
            self.request(storage_updates.unwrap()).await;
        }
    }

    ///
    /// Performs a path edit on a layer
    ///
    pub fn path_edit<'a>(&'a mut self, layer_id: u64, when: Duration, edit: &'a PathEdit) -> impl 'a+Future<Output=()> {
        async move {
            use self::PathEdit::*;

            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return; }
                Some(keyframe)  => keyframe
            };

            match edit {
                CreatePath(element_id, components)      => {
                    let element_id = *element_id;

                    // Need to have the brush definition and properties defined for the current path
                    let (defn, props) = if let (Some(defn), Some(props)) = (&self.path_brush_defn, &self.path_brush_props) {
                        (defn.clone(), props.clone())
                    } else {
                        // No properties set
                        return;
                    };

                    // Create the path element
                    let element = PathElement::new(element_id, Path::from_elements(components.iter().cloned()), defn.clone(), props.clone());
                    let element = Vector::Path(element);

                    // Edit the keyframe
                    let storage_updates = current_keyframe.future(move |current_keyframe| {
                        async move {
                            // Add to a wrapper
                            let wrapper         = ElementWrapper::with_element(element, when);

                            // Append to the current keyframe and return the list of storage commands
                            let mut add_element = current_keyframe.add_element_to_end(element_id, wrapper);

                            // Make sure the definition and properties are attached to the keyframe so the path can find them later on
                            add_element.push(StorageCommand::AttachElementToLayer(layer_id, defn.id().id().unwrap_or(0), when));
                            add_element.push(StorageCommand::AttachElementToLayer(layer_id, props.id().id().unwrap_or(0), when));

                            add_element
                        }.boxed()
                    }).await;

                    // Send to the storage
                    self.request(storage_updates.unwrap()).await;
                }

                SelectBrush(element_id, defn, style)    => {
                    // Create a brush definition element
                    let defn                = BrushDefinitionElement::new(*element_id, defn.clone(), *style);
                    self.path_brush_defn    = Some(Arc::new(defn.clone()));

                    // Save as an element (it gets attached to a frame when used in a path)
                    let element             = Vector::BrushDefinition(defn);
                    let element_id          = element_id.id().unwrap_or(0);
                    let element_wrapper     = ElementWrapper::with_element(element, when);

                    let mut element_string  = String::new();
                    element_wrapper.serialize(&mut element_string);

                    self.request(vec![StorageCommand::WriteElement(element_id, element_string)]).await;
                }

                BrushProperties(element_id, properties) => {
                    // Create a brush properties element
                    let defn                = BrushPropertiesElement::new(*element_id, properties.clone());
                    self.path_brush_props    = Some(Arc::new(defn.clone()));

                    // Save as an element
                    let element             = Vector::BrushProperties(defn);
                    let element_id          = element_id.id().unwrap_or(0);
                    let element_wrapper     = ElementWrapper::with_element(element, when);

                    let mut element_string  = String::new();
                    element_wrapper.serialize(&mut element_string);

                    self.request(vec![StorageCommand::WriteElement(element_id, element_string)]).await;
                }
            };
        }
    }

    ///
    /// Updates a one or more elements via an update function
    ///
    pub fn update_elements<'a, UpdateFn>(&'a mut self, element_ids: Vec<i64>, update_fn: UpdateFn) -> impl 'a+Future<Output=()>
    where UpdateFn: 'a+Fn(ElementWrapper) -> ElementWrapper {
        async move {
            // Update the elements that are returned
            let mut updates = vec![];

            // Build a hashset of the remaining elements
            let mut remaining = element_ids.iter().cloned().collect::<HashSet<_>>();

            // ... until we've removed all the remaining elements...
            while let Some(root_element) = remaining.iter().nth(0).cloned() {
                // Fetch the keyframe that the root element is in
                let keyframe_response = self.request_one(StorageCommand::ReadElementAttachments(root_element)).await;

                if let Some(StorageResponse::ElementAttachments(_elem, mut keyframes)) = keyframe_response  {
                    if let Some((layer_id, keyframe_time)) = keyframes.pop() {
                        // Need to retrieve the keyframe (some elements depend on others, so we need the whole keyframe)
                        // Most of the time we'll be editing a single frame so this won't be too expensive
                        if let Some(keyframe) = self.edit_keyframe(layer_id, keyframe_time).await {
                            // ... the element is in a keyframe, and we retrieved that keyframe
                            let to_process = remaining.iter().cloned().collect::<Vec<_>>();

                            for element_id in to_process {
                                // Try to retrieve the element from the keyframe
                                let existing_element = keyframe.future(move |keyframe| {
                                    async move {
                                        let elements = keyframe.elements.lock().unwrap();
                                        elements.get(&ElementId::Assigned(element_id)).cloned()
                                    }.boxed()
                                }).await;

                                // Update the existing element if we managed to retrieve it
                                if let Ok(Some(existing_element)) = existing_element {
                                    // Process via the update function
                                    let updated_element = update_fn(existing_element);

                                    // Generate the update of the serialized element
                                    let mut serialized  = String::new();
                                    updated_element.serialize(&mut serialized);

                                    updates.push(StorageCommand::WriteElement(element_id, serialized));

                                    // Replace the element in the keyframe
                                    keyframe.desync(move |keyframe| {
                                        keyframe.elements.lock().unwrap()
                                            .insert(ElementId::Assigned(element_id), updated_element);
                                    });

                                    // Remove the element from the remaining list so we don't try to update it again
                                    remaining.remove(&element_id);
                                }
                            }
                        }
                    }
                }

                // The root element is always removed from the remaining list even if we couldn't get its keyframe
                remaining.remove(&root_element);
            }

            // Update all of the elements
            self.request(updates).await;
        }
    }

    ///
    /// Performs an element edit on this animation
    ///
    pub fn element_edit<'a>(&'a mut self, element_ids: &'a Vec<ElementId>, element_edit: &'a ElementEdit) -> impl 'a+Future<Output=()> {
        async move {
            use self::ElementEdit::*;

            let element_ids = element_ids.iter().map(|elem| elem.id()).flatten().collect();

            match element_edit {
                AddAttachment(attach_id)        => { self.update_elements(element_ids, |mut wrapper| { wrapper.attachments.push(*attach_id); wrapper }).await; }
                RemoveAttachment(attach_id)     => { self.update_elements(element_ids, |mut wrapper| { wrapper.attachments.retain(|id| id != attach_id); wrapper }).await; }
                SetControlPoints(new_points)    => { self.update_elements(element_ids, |mut wrapper| { wrapper.element = wrapper.element.with_adjusted_control_points(new_points.clone()); wrapper }).await; }
                SetPath(new_path)               => { }
                Order(ordering)                 => { }
                Delete                          => { }
                DetachFromFrame                 => { }
            }
        }
    }

    ///
    /// Performs a motion edit on this animation
    ///
    pub fn motion_edit<'a>(&'a mut self, motion_id: ElementId, motion_edit: &'a MotionEdit) -> impl 'a+Future<Output=()> {
        async move {

        }
    }

    ///
    /// Adds a key frame to a layer
    ///
    pub fn add_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=()> { 
        async move {
            self.request_one(StorageCommand::AddKeyFrame(layer_id, when)).await;
        } 
    }

    ///
    /// Removes a key frame from a layer
    ///
    pub fn remove_key_frame<'a>(&'a mut self, layer_id: u64, when: Duration) -> impl 'a+Future<Output=()> { 
        async move {
            self.request_one(StorageCommand::DeleteKeyFrame(layer_id, when)).await;
        } 
    }

    ///
    /// Sets the name of a layer
    ///
    pub fn set_layer_name<'a>(&'a mut self, layer_id: u64, name: &'a str) -> impl 'a+Future<Output=()> { 
        async move {
            // Read the current properties for this layer
            let mut properties = match self.request_one(StorageCommand::ReadLayerProperties(layer_id)).await {
                Some(StorageResponse::LayerProperties(_, properties)) => {
                    LayerProperties::deserialize(&mut properties.chars())
                        .unwrap_or_else(|| LayerProperties::default())
                }

                _ => LayerProperties::default()
            };

            // Update the name
            properties.name = name.to_string();

            // Save back to the storage
            let mut serialized = String::new();
            properties.serialize(&mut serialized);
            self.request_one(StorageCommand::WriteLayerProperties(layer_id, serialized)).await;
        } 
    }

    ///
    /// Sets the order of a layer (which is effectively the ID of the layer this layer should appear behind)
    ///
    pub fn set_layer_ordering<'a>(&'a mut self, layer_id: u64, ordering: u32) -> impl 'a+Future<Output=()> {
        async move { 
            self.request_one(StorageCommand::OrderLayer(layer_id, ordering as u64)).await;
        } 
    }

    ///
    /// Adds a new layer with a particular ID to this animation
    ///
    pub fn add_new_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {
            self.request_one(StorageCommand::AddLayer(layer_id)).await;
        }
    }

    ///
    /// Removes the layer with the specified ID from the animation
    ///
    pub fn remove_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {
            self.request_one(StorageCommand::DeleteLayer(layer_id)).await;
        }
    }
}
