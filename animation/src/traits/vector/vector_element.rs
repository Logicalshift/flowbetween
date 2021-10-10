use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;
use flo_canvas_animation::*;

use std::time::Duration;
use std::sync::*;
use std::any::*;

///
/// Represents a vector element in a frame
///
pub trait VectorElement : Send+Any {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId;

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId);

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>>;

    ///
    /// Renders this vector element to an animated layer
    ///
    fn render_animated(&self, gc: &mut AnimationLayerContext<'_>, properties: &VectorProperties, when: Duration) { self.render_static(gc, properties, when); }

    ///
    /// Renders this vector element as a static element in a graphics context
    ///
    /// This can be used for rendering a preview of this element, such as when editing it
    ///
    fn render_static(&self, gc: &mut dyn GraphicsContext, properties: &VectorProperties, when: Duration);

    ///
    /// Returns a selection priority if the specified point should select this element when clicked using the selection tool
    ///
    /// The return value is 'None' if the point is not wihtin this element, otherwise is the selection priority. Items
    /// with higher priorities are selected ahead of items with lower priorities. `Some(1)` is the default priority,
    /// with `Some(0)` indicating a point that is within the bounding box for the element but not within the element
    /// itself. 
    ///
    fn is_selected_with_point(&self, properties: &VectorProperties, x: f64, y: f64) -> Option<i32> {
        if let Some(paths) = self.to_path(properties, PathConversion::RemoveInteriorPoints) {
            // Check if the specified point is inside one of the paths
            let mut priority = None;

            for path in paths.iter() {
                let mut path_priority = None;

                let bounds = Rect::from(path); 
                if x >= bounds.x1 as _ && x <= bounds.x2 as _ && y >= bounds.y1 as _ && y <= bounds.y2 as _ {
                    // Is within the bounding box
                    path_priority = Some(0);

                    // TODO: Check if we're within the path itself to see if we want to increase the priority to 1
                }

                // If this path is higher priority than the current priority, use it as a target
                if priority.is_none() || path_priority > priority {
                    priority = path_priority;
                }
            }

            priority
        } else {
            // Not within the path
            None
        }
    }

    ///
    /// For elements that are not visible in the final animation, renders an editing overlay to the specified graphics context 
    ///
    fn render_overlay(&self, _gc: &mut dyn GraphicsContext, _when: Duration) { }

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { properties }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint>;

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector;
}
