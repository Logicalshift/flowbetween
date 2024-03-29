use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing a path definition
///
#[derive(Clone, PartialEq, Debug)]
pub struct PathElement {
    /// The ID of this path
    id: ElementId,

    /// The components that make up this path
    path: Path
}

impl PathElement {
    ///
    /// Creates a new path element with the specified properties
    ///
    pub fn new(id: ElementId, path: Path) -> PathElement {
        PathElement {
            id,
            path
        }
    }

    ///
    /// Returns a reference to the underlying path for this element
    ///
    pub fn path(&self) -> &Path {
        &self.path
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
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.id = new_id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>> {
        // Final result depends on the options that are set
        let path = match options {
            PathConversion::Fastest                 => Some(vec![self.path.clone()]),
            PathConversion::RemoveInteriorPoints    => {
                let subpaths    = self.path.to_subpaths();
                let path        = path_remove_overlapped_points(&subpaths, 0.01);
                let path        = Path::from_paths(&path);
                Some(vec![path])
            }
        };

        // Apply any transformations in the properties
        path.map(|path| {
            let mut path = path;

            for path_component in path.iter_mut() {
                path_component.apply_transformations(properties);
            }

            path
        })
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> {
        properties
    }

    ///
    /// Renders this vector element
    ///
    fn render_static(&self, gc: &mut dyn GraphicsContext, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));

        if properties.transformations.len() > 0 {
            // Transform the path
            let mut path = self.path.clone();
            path.apply_transformations(properties);

            // Draw the transformed path
            gc.winding_rule(WindingRule::EvenOdd);
            gc.draw_list(properties.brush.render_path(&properties.brush_properties, &path));
        } else {
            // No transformations: just render the path directly
            gc.draw_list(properties.brush.render_path(&properties.brush_properties, &self.path));
        }
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint> {
        self.path.elements_ref()
            .flat_map(|component| {
                match component {
                    PathComponent::Move(pos)                => vec![ControlPoint::BezierPoint(pos.x() as f64, pos.y() as f64)],
                    PathComponent::Line(pos)                => vec![ControlPoint::BezierPoint(pos.x() as f64, pos.y() as f64)],
                    PathComponent::Bezier(pos, cp1, cp2)    => vec![
                        ControlPoint::BezierControlPoint(cp1.x() as f64, cp1.y() as f64),
                        ControlPoint::BezierControlPoint(cp2.x() as f64, cp2.y() as f64),
                        ControlPoint::BezierPoint(pos.x() as f64, pos.y() as f64)
                    ],
                    PathComponent::Close                    => vec![]
                }
            })
            .map(|control_point| properties.transform_control_point(&control_point))
            .collect()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector {
        let inverse_properties = properties.with_inverse_transformation().unwrap_or_else(|| properties.clone());

        // Iterator for fetching points from
        let mut next_position = new_positions.into_iter()
            .map(|(x, y)| inverse_properties.transform_point(&Coord2(x as f64, y as f64)))
            .map(|Coord2(x, y)| PathPoint::new(x as f32, y as f32));

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
            id:     self.id,
            path:   Path::from_elements_arc(Arc::new(new_elements))
        })
    }
}

impl Into<Vector> for PathElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::Path(self)
    }
}
