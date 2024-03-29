use super::super::edit::*;
use super::super::brush::*;
use super::super::vector::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use flo_curves::*;
use flo_curves::bezier::path::*;
use flo_canvas::*;
use flo_canvas_animation::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents the active properties for a vector layer
///
/// Elements can update the properties, which persist to the next element.
/// This saves some space in that properties don't need to be encoded with
/// each element.
///
#[derive(Clone)]
pub struct VectorProperties {
    /// The active brush
    pub brush: Arc<dyn Brush>,

    /// The properties set for the active brush
    pub brush_properties: BrushProperties,

    /// Transformations to apply to the element during rendering
    pub transformations: Arc<Vec<Transformation>>,

    /// Returns the 
    pub retrieve_attachments: Arc<dyn (Fn(ElementId) -> Vec<Vector>) + Sync+Send>,

    /// Provides an override for how a vector element is rendered
    pub render_vector_static: Arc<dyn (Fn(&mut dyn GraphicsContext, Vector, Duration, &VectorProperties)) + Sync+Send>,

    /// Provides an override for how a vector element is rendered
    pub render_vector_animated: Arc<dyn (Fn(&mut AnimationLayerContext<'_>, Vector, Duration, &VectorProperties)) + Sync+Send>
}

impl VectorProperties {
    ///
    /// Creates the default brush properties
    ///
    pub fn default() -> VectorProperties {
        VectorProperties {
            brush:                  Arc::new(InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw)),
            brush_properties:       BrushProperties::new(),
            transformations:        Arc::new(vec![]),
            retrieve_attachments:   Arc::new(|_| vec![]),
            render_vector_static:   Arc::new(|gc, vector, when, properties| vector.render_static(gc, properties, when)),
            render_vector_animated: Arc::new(|gc, vector, when, properties| vector.render_animated(gc, properties, when))
        }
    }

    ///
    /// Prepares the context to render with these properties
    ///
    pub fn prepare_to_render(&self, gc: &mut dyn GraphicsContext) {
        gc.draw_list(self.brush.prepare_to_render(&self.brush_properties));
    }

    ///
    /// Renders the specified element with these properties
    ///
    pub fn render_animated(&self, gc: &mut AnimationLayerContext<'_>, element: Vector, when: Duration) {
        // Render this element
        (self.render_vector_animated)(gc, element, when, self);
    }

    ///
    /// Renders the specified element with these properties
    ///
    pub fn render_static(&self, gc: &mut dyn GraphicsContext, element: Vector, when: Duration) {
        // Render this element
        (self.render_vector_static)(gc, element, when, self);
    }

    ///
    /// Transforms a control point via the transformations specified in these properties
    ///
    pub fn transform_control_point(&self, control_point: &ControlPoint) -> ControlPoint {
        let mut result = control_point.clone();

        for transform in self.transformations.iter() {
            result = transform.transform_control_point(&result);
        }

        result
    }

    ///
    /// Transforms a coordinate via the transformations specified in these properties
    ///
    pub fn transform_point<Coord>(&self, coord: &Coord) -> Coord
    where Coord: Coordinate {
        let mut result = coord.clone();

        for transform in self.transformations.iter() {
            result = transform.transform_point(&result);
        }

        result
    }

    ///
    /// Applies the transformation to a path implementing the path factory interface from flo_curves
    ///
    pub fn transform_bezier_path<P: BezierPathFactory>(&self, path: P) -> P {
        let mut result = path;

        for transform in self.transformations.iter() {
            result = transform.transform_bezier_path(&result);
        }

        result
    }

    ///
    /// Creates a variant of these properties with the transformations inverted
    ///
    pub fn with_inverse_transformation(&self) -> Option<VectorProperties> {
        // Reverse the transformations and invert each one
        let inverted_transformations = self.transformations.iter()
            .rev()
            .map(|transform| transform.invert())
            .collect::<Option<Vec<_>>>()?;

        // Generate the result
        Some(VectorProperties {
            brush:                  Arc::clone(&self.brush),
            brush_properties:       self.brush_properties.clone(),
            transformations:        Arc::new(inverted_transformations),
            retrieve_attachments:   Arc::clone(&self.retrieve_attachments),
            render_vector_static:   Arc::clone(&self.render_vector_static),
            render_vector_animated: Arc::clone(&self.render_vector_animated)
        })
    }
}
