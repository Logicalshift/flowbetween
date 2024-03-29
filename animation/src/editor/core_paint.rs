use super::element_wrapper::*;
use super::stream_animation_core::*;
use crate::undo::*;
use crate::traits::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Performs a paint edit on a layer
    ///
    pub fn paint_edit<'a>(&'a mut self, layer_id: u64, when: Duration, edit: &'a PaintEdit) -> impl 'a+Future<Output=ReversedEdits> { 
        async move { 
            use self::PaintEdit::*;

            let mut reversed_edits = ReversedEdits::new();

            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return ReversedEdits::empty(); }
                Some(keyframe)  => keyframe
            };

            self.cached_layers.remove(&layer_id);

            let (id, wrapper) = match edit {
                SelectBrush(element_id, defn, style)    => {
                    // Create a brush definition element
                    let defn            = BrushDefinitionElement::new(*element_id, defn.clone(), *style);
                    let element         = Vector::BrushDefinition(defn);
                    let element_id      = element_id.id().unwrap_or(0);
                    let wrapper         = ElementWrapper::unattached_with_element(element, when);

                    self.brush_defn     = Some(ElementId::Assigned(element_id));

                    (element_id, Some(wrapper))
                }

                BrushProperties(element_id, properties) => {
                    // Create a brush properties element
                    let defn            = BrushPropertiesElement::new(*element_id, properties.clone());
                    let element         = Vector::BrushProperties(defn);
                    let element_id      = element_id.id().unwrap_or(0);
                    let wrapper         = ElementWrapper::unattached_with_element(element, when);
                    self.brush_props    = Some(ElementId::Assigned(element_id));

                    (element_id, Some(wrapper))
                }

                BrushStroke(element_id, points)         => {
                    // Create a brush stroke element, using the current brush for the layer
                    let active_brush    = current_keyframe.future_sync(|keyframe| async move { keyframe.get_active_brush() }.boxed()).await.unwrap();
                    let points          = active_brush.brush_points_for_raw_points(points);
                    let brush_element   = BrushElement::new(*element_id, Arc::new(points));
                    let element         = Vector::BrushStroke(brush_element);
                    let element_id      = element_id.id().unwrap_or(0);
                    let mut wrapper     = ElementWrapper::attached_with_element(element, when);

                    wrapper.attachments = vec![self.brush_defn, self.brush_props].into_iter().flatten().collect();

                    (element_id, Some(wrapper))
                }

                CreateShape(element_id, width, shape)          => {
                    // Create a shape element, using the current brush for the layer
                    let shape_element   = ShapeElement::new(*element_id, *width as f64, shape.clone());
                    let element         = Vector::Shape(shape_element);
                    let element_id      = element_id.id().unwrap_or(0);
                    let mut wrapper     = ElementWrapper::attached_with_element(element, when);

                    wrapper.attachments = vec![self.brush_defn, self.brush_props].into_iter().flatten().collect();

                    (element_id, Some(wrapper))
                }

                Fill(element_id, point, options)        => {
                    let element_id = element_id.id().unwrap_or(0);
                    self.paint_fill(layer_id, when, ElementId::Assigned(element_id), *point, options).await;

                    // We don't return a wrapper from here so add the element delete manually
                    reversed_edits.push(AnimationEdit::Element(vec![ElementId::Assigned(element_id)], ElementEdit::Delete));
                    (element_id, None)
                }
            };

            if let Some(wrapper) = wrapper {
                // Edit the keyframe
                let storage_updates = current_keyframe.future_sync(move |current_keyframe| {
                    async move {
                        // Append to the current keyframe and return the list of storage commands
                        current_keyframe.add_element_to_end(ElementId::Assigned(id), wrapper)
                    }.boxed()
                }).await;

                // Send to the storage
                self.request(storage_updates.unwrap()).await;

                // Reversing this means deleting the new element
                reversed_edits.push(AnimationEdit::Element(vec![ElementId::Assigned(id)], ElementEdit::Delete));
            }

            reversed_edits
        }
    }
}
