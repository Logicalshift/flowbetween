use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;
use crate::raycast::*;

use flo_canvas::*;
use flo_canvas_animation::*;
use flo_canvas_animation::description::*;

use std::fmt;
use std::iter;
use std::sync::*;
use std::time::{Duration};

pub const ANIMATION_OUTLINE:                Color = Color::Rgba(0.2, 0.8, 0.2, 0.6);
pub const ANIMATION_OUTLINE_DARK:           Color = Color::Rgba(0.1, 0.4, 0.1, 0.4);

///
/// Represents an animation region element
///
pub struct AnimationElement {
    id:             ElementId, 
    description:    RegionDescription, 
    cached_region:  Mutex<Option<Arc<dyn AnimationRegion>>>
}

impl AnimationElement {
    ///
    /// Creates a new animation element
    ///
    pub fn new(id: ElementId, description: RegionDescription) -> AnimationElement {
        AnimationElement {
            id:             id,
            description:    description,
            cached_region:  Mutex::new(None)
        }
    }

    ///
    /// The description for this animation element
    ///
    pub fn description<'a>(&'a self) -> &'a RegionDescription {
        &self.description
    }

    ///
    /// The animation region for this animation element
    ///
    pub fn animation_region(&self) -> Arc<dyn AnimationRegion> {
        let region = {
            let mut cached_region = self.cached_region.lock().unwrap();

            if let Some(region) = &*cached_region {
                // Use the existing region
                Arc::clone(region)
            } else {
                // Cache a new region
                let region      = (&self.description).into();
                *cached_region  = Some(Arc::clone(&region));
                region
            }
        };

        region
    }
}

impl Clone for AnimationElement {
    fn clone(&self) -> Self {
        AnimationElement {
            id:             self.id,
            description:    self.description.clone(),
            cached_region:  Mutex::new(self.cached_region.lock().unwrap().clone())
        }
    }
}

impl fmt::Debug for AnimationElement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("AnimationRegion({:?}, {:?})", self.id, self.description))
    }
}

impl VectorElement for AnimationElement {
    fn id(&self) -> ElementId { 
        self.id 
    }

    fn set_id(&mut self, new_id: ElementId) { 
        self.id = new_id; 
    }

    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> { 
        None
    }

    fn render_animated(&self, gc: &mut AnimationLayerContext<'_>, properties: &VectorProperties, when: Duration) { 
        gc.add_region(self.animation_region());
        self.render_static(gc, properties, when);
    }

    fn is_selected_with_point(&self, _properties: &VectorProperties, x: f64, y: f64) -> Option<i32> {
        let path                    = &self.description.0;
        let (collided, distance)    = point_is_in_path_with_distance(path, &Point2D(x, y));

        if collided {
            if distance < 2.0 {
                // Clicking on the edge of the animation region selects it
                Some(150)
            } else if distance < 8.0 {
                // Clicking close to the edge of the animation region selects it if no other elements are near
                Some(90)
            } else {
                // Bounding boxes of other elements take priority over the region
                Some(-150)
            }
        } else {
            if distance < 2.0 {
                // Clicking on the edge of the animation region selects it
                Some(150)
            } else {
                None
            }
        }
    }

    fn render_overlay(&self, gc: &mut dyn GraphicsContext, _when: Duration) { 
        gc.new_path();

        // Add the region outline to the paths
        let RegionDescription(paths, _effect) = &self.description;
        for BezierPath(start_point, other_points) in paths.iter() {
            // Add closed paths for each section of the region
            gc.move_to(start_point.x() as _, start_point.y() as _);
            for BezierPoint(cp1, cp2, end_point) in other_points.iter() {
                gc.bezier_curve_to(
                    end_point.x() as _, end_point.y() as _,
                    cp1.x() as _, cp1.y() as _,
                    cp2.x() as _, cp2.y() as _);
            }
            gc.close_path();
        }

        // Draw the region outline
        gc.line_width_pixels(3.0);
        gc.stroke_color(ANIMATION_OUTLINE_DARK);
        gc.stroke();

        gc.line_width_pixels(1.0);
        gc.stroke_color(ANIMATION_OUTLINE);
        gc.stroke();

        // TODO: add any motion paths that might be present in the description
    }

    fn render_static(&self, _gc: &mut dyn GraphicsContext, _properties: &VectorProperties, _when: Duration) { 
    }

    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { 
        properties 
    }

    fn control_points(&self, _properties: &VectorProperties) -> Vec<ControlPoint> {
        // The 'normal' control points are the points in the main bezier path
        let RegionDescription(paths, _effect)        = &self.description;

        let regions = paths.iter()
            .flat_map(|BezierPath(start_point, other_points)| {
                iter::once(ControlPoint::BezierPoint(start_point.x(), start_point.y()))
                    .chain(other_points.iter().flat_map(|BezierPoint(cp1, cp2, end_point)| [
                        ControlPoint::BezierControlPoint(cp1.x(), cp1.y()),
                        ControlPoint::BezierControlPoint(cp2.x(), cp2.y()),
                        ControlPoint::BezierPoint(end_point.x(), end_point.y())
                    ]))
            });

        // TODO: get the control points from the effect

        regions.collect()
    }

    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        // Create the new paths by mapping the points from the new positions onto the existing paths
        let mut region_outline  = vec![];
        let mut pos_iter        = new_positions.into_iter();

        for old_path in self.description.0.iter() {
            // Update the region control points
            let start_point         = pos_iter.next().unwrap().into();
            let mut outline_points  = vec![];

            for _ in 0..(old_path.1.len()) {
                let cp1         = pos_iter.next().unwrap().into();
                let cp2         = pos_iter.next().unwrap().into();
                let end_point   = pos_iter.next().unwrap().into();

                outline_points.push(BezierPoint(cp1, cp2, end_point));
            }

            region_outline.push(BezierPath(start_point, outline_points));
        }

        // TODO: update the effect control points

        // Create the new region description
        let effect              = self.description.1.clone();
        let region_description  = RegionDescription(region_outline, effect);

        Vector::AnimationRegion(AnimationElement::new(self.id, region_description))
    }
}
