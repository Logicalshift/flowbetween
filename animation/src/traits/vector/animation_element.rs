use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::transformation::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;
use crate::raycast::*;

use flo_canvas::*;
use flo_canvas_animation::*;
use flo_canvas_animation::description::*;
use flo_curves::bezier::path::{SimpleBezierPath};

use std::fmt;
use std::iter;
use std::sync::*;
use std::time::{Duration};

pub const ANIMATION_OUTLINE:                Color = Color::Rgba(0.2, 0.8, 0.2, 0.6);
pub const ANIMATION_OUTLINE_DARK:           Color = Color::Rgba(0.1, 0.4, 0.1, 0.4);

///
/// Represents an animation region whose base region has had a transformation applied to its perimeter
///
struct TransformedAnimationRegion {
    /// The region that has been transformed
    region: Arc<dyn AnimationRegion>,

    /// The transformation that was applied to the region
    transformation: Arc<Vec<Transformation>>
}

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

    fn to_path(&self, properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> { 
        // Convert the region path to pathops
        let drawing = self.description.0
            .iter()
            .map(|path| properties.transform_bezier_path(path.clone()))
            .flat_map(|BezierPath(start_point, curves)| {
                use self::PathOp::*;

                iter::once(Move(start_point.x() as _, start_point.y() as _))
                    .chain(curves.into_iter()
                        .map(|BezierPoint(cp1, cp2, ep)| BezierCurve(((cp1.x() as _, cp1.y() as _), (cp2.x() as _, cp2.y() as _)), (ep.x() as _, ep.y() as _)) ))
                    .chain(iter::once(PathOp::ClosePath))
            })
            .collect::<Vec<_>>();

        // Convert the pathops to a path
        Some(vec![drawing.into()])
    }

    fn render_animated(&self, gc: &mut AnimationLayerContext<'_>, properties: &VectorProperties, when: Duration) { 
        let mut region = self.animation_region();

        if properties.transformations.len() > 0 {
            region = Arc::new(TransformedAnimationRegion { 
                region: region, 
                transformation: Arc::clone(&properties.transformations)
            });
        }

        gc.add_region(region);
        self.render_static(gc, properties, when);
    }

    fn is_selected_with_point(&self, properties: &VectorProperties, x: f64, y: f64) -> Option<i32> {
        let path                    = &self.description.0;
        let path                    = path.iter().map(|path| properties.transform_bezier_path(path.clone())).collect();
        let (collided, distance)    = point_is_in_path_with_distance(&path, &Point2D(x, y));

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
        // TODO: these should be generated from the attachments for this element
        let properties = VectorProperties::default();

        gc.new_path();

        // Add the region outline to the paths
        let RegionDescription(paths, _effect)   = &self.description;
        let paths                               = paths.iter().map(|path| properties.transform_bezier_path(path.clone()));
        for BezierPath(start_point, other_points) in paths {
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

    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint> {
        // The 'normal' control points are the points in the main bezier path
        let RegionDescription(paths, _effect)   = &self.description;
        let paths                               = paths.iter().map(|path| properties.transform_bezier_path(path.clone()));

        let regions = paths
            .flat_map(|BezierPath(start_point, other_points)| {
                iter::once(ControlPoint::BezierPoint(start_point.x(), start_point.y()))
                    .chain(other_points.into_iter().flat_map(|BezierPoint(cp1, cp2, end_point)| [
                        ControlPoint::BezierControlPoint(cp1.x(), cp1.y()),
                        ControlPoint::BezierControlPoint(cp2.x(), cp2.y()),
                        ControlPoint::BezierPoint(end_point.x(), end_point.y())
                    ]))
            });

        // TODO: get the control points from the effect

        regions.collect()
    }

    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector {
        // Create the new paths by mapping the points from the new positions onto the existing paths
        let inverse_properties  = properties.with_inverse_transformation().expect("Invertable transformation");
        let mut region_outline  = vec![];
        let mut pos_iter        = new_positions.into_iter()
            .map(|(x, y)| inverse_properties.transform_point(&Coord2(x as _, y as _)))
            .map(|Coord2(x, y)| (x, y));

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

impl AnimationEffect for TransformedAnimationRegion {
    ///
    /// Returns the duration of this effect (or None if this effect will animate forever)
    ///
    /// If the effect is passed a time that's after where the 'duration' has completed it should always generate the same result
    ///
    fn duration(&self) -> Option<f64> {
        self.region.duration()
    }

    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        self.region.animate(region_contents, time)
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        self.region.animate_cached(region_contents)
    }
}

impl AnimationRegion for TransformedAnimationRegion {
    ///
    /// Returns the definition of a sub-region that this animation will affect from the static layer
    ///
    /// This will return the location of the region at a particular time so that drawing added after
    /// the initial keyframe can be incorporated into the appropriate region
    ///
    fn region(&self, time: Duration) -> Vec<SimpleBezierPath> {
        let original_path = self.region.region(time);

        original_path.into_iter()
            .map(|path| {
                let mut path = path;

                for transform in self.transformation.iter() {
                    path = transform.transform_bezier_path(&path);
                }

                path
            })
            .collect()
    }
}
