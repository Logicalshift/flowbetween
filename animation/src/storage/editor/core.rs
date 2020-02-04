use super::super::storage_api::*;
use super::super::file_properties::*;
use super::super::super::traits::*;

use flo_stream::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

pub (super) struct StreamAnimationCore {
    /// Stream where responses to the storage requests are sent
    pub (super) storage_responses: BoxStream<'static, Vec<StorageResponse>>,

    /// Publisher where we can send requests for storage actions
    pub (super) storage_requests: Publisher<Vec<StorageCommand>>,
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
    /// Performs a set of edits on the core
    ///
    pub fn perform_edits<'a>(&'a mut self, edits: Arc<Vec<AnimationEdit>>) -> impl 'a+Future<Output=()> {
        async move {
            // Process the edits in the order that they arrive
            for edit in edits.iter() {
                use self::AnimationEdit::*;

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
    pub fn layer_edit<'a>(&'a mut self, layer_id: u64, layer_edit: &LayerEdit) -> impl 'a+Future<Output=()> {
        async move {

        }
    }

    ///
    /// Performs an element edit on this animation
    ///
    pub fn element_edit<'a>(&'a mut self, element_ids: &Vec<ElementId>, layer_edit: &ElementEdit) -> impl 'a+Future<Output=()> {
        async move {

        }
    }

    ///
    /// Performs a motion edit on this animation
    ///
    pub fn motion_edit<'a>(&'a mut self, motion_id: ElementId, motion_edit: &MotionEdit) -> impl 'a+Future<Output=()> {
        async move {

        }
    }

    ///
    /// Adds a new layer with a particular ID to this animation
    ///
    pub fn add_new_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {

        }
    }

    ///
    /// Removes the layer with the specified ID from the animation
    ///
    pub fn remove_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {

        }
    }
}
