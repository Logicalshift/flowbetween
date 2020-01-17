use super::element::*;
use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::brush_definition_element::*;
use super::brush_properties_element::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing a path definition
///
#[derive(Clone, Debug)]
pub struct PathElement {
    /// The ID of this path
    id: ElementId,

    /// The components that make up this path
    path: Path,

    /// The brush to use for this path
    brush: Arc<BrushDefinitionElement>,

    /// The properties to use for this path
    brush_properties: Arc<BrushPropertiesElement>
}

impl PathElement {
    ///
    /// Creates a new path element with the specified properties
    ///
    pub fn new(id: ElementId, path: Path, brush: Arc<BrushDefinitionElement>, brush_properties: Arc<BrushPropertiesElement>) -> PathElement {
        PathElement {
            id,
            path,
            brush,
            brush_properties
        }
    }

    ///
    /// Returns a reference to the underlying path for this element
    ///
    pub fn path(&self) -> &Path {
        &self.path
    }

    ///
    /// Returns the brush definition for this path element
    ///
    pub fn brush(&self) -> Arc<BrushDefinitionElement> {
        Arc::clone(&self.brush)
    }

    ///
    /// Returns the properties for this path element
    ///
    pub fn properties(&self) -> Arc<BrushPropertiesElement> {
        Arc::clone(&self.brush_properties)
    }
}

impl VectorElement for PathElement {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> {
        Some(vec![self.path.clone()])
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        let properties = self.brush.update_properties(properties);
        let properties = self.brush_properties.update_properties(properties);

        properties
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
        gc.draw_list(properties.brush.render_path(&properties.brush_properties, &self.path));
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        // Gather all the points in this path in one place
        let all_points          = self.path.elements_ref()
            .flat_map(|component| {
                match component {
                    PathComponent::Move(pos)                => vec![pos],
                    PathComponent::Line(pos)                => vec![pos],
                    PathComponent::Bezier(pos, cp1, cp2)    => vec![cp1, cp2, pos],
                    PathComponent::Close                    => vec![]
                }
            });

        // Transform the points
        let transformed_points  = motion.transform_path_points(when, all_points);

        // Collect into a set of new elements
        let mut next_position   = transformed_points;
        let new_elements        = self.path.elements_ref()
            .map(|component| {
                match component {
                    PathComponent::Move(_)          => PathComponent::Move(next_position.next().unwrap()),
                    PathComponent::Line(_)          => PathComponent::Line(next_position.next().unwrap()),
                    PathComponent::Bezier(_, _, _)  => {
                        let cp1 = next_position.next().unwrap();
                        let cp2 = next_position.next().unwrap();
                        let pos = next_position.next().unwrap();

                        PathComponent::Bezier(pos, cp1, cp2)
                    },
                    PathComponent::Close            => PathComponent::Close
                }
            })
            .collect();

        // Create a new path transformed with these points
        Vector::Path(PathElement {
            id:                 self.id,
            brush:              Arc::clone(&self.brush),
            brush_properties:   Arc::clone(&self.brush_properties),
            path:               Path::from_elements_arc(Arc::new(new_elements))
        })
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        self.path.elements_ref()
            .flat_map(|component| {
                match component {
                    PathComponent::Move(pos)                => vec![ControlPoint::BezierPoint(pos.x(), pos.y())],
                    PathComponent::Line(pos)                => vec![ControlPoint::BezierPoint(pos.x(), pos.y())],
                    PathComponent::Bezier(pos, cp1, cp2)    => vec![
                        ControlPoint::BezierControlPoint(cp1.x(), cp1.y()),
                        ControlPoint::BezierControlPoint(cp2.x(), cp2.y()),
                        ControlPoint::BezierPoint(pos.x(), pos.y())
                    ],
                    PathComponent::Close                    => vec![]
                }
            })
            .collect()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        // Iterator for fetching points from
        let mut next_position = new_positions.into_iter()
            .map(|(x, y)| PathPoint::new(x, y));

        // Transform the components to generate a new path
        let new_elements = self.path.elements_ref()
            .map(|component| {
                match component {
                    PathComponent::Move(_)          => PathComponent::Move(next_position.next().unwrap()),
                    PathComponent::Line(_)          => PathComponent::Line(next_position.next().unwrap()),
                    PathComponent::Bezier(_, _, _)  => {
                        let cp1 = next_position.next().unwrap();
                        let cp2 = next_position.next().unwrap();
                        let pos = next_position.next().unwrap();

                        PathComponent::Bezier(pos, cp1, cp2)
                    },
                    PathComponent::Close            => PathComponent::Close
                }
            })
            .collect();

        Vector::Path(PathElement {
            id:                 self.id,
            brush:              Arc::clone(&self.brush),
            brush_properties:   Arc::clone(&self.brush_properties),
            path:               Path::from_elements_arc(Arc::new(new_elements))
        })
    }
}

impl Into<Vector> for PathElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::Path(self)
    }
}
