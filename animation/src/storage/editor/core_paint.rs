use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::super::traits::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
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
}
