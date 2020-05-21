use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::brush_definition_element::*;
use super::brush_properties_element::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::path::*;

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
            let path = if properties.transformations.len() > 0 {
                let mut path = path;

                for transform in properties.transformations.iter() {
                    for path_component in path.iter_mut() {
                        *path_component = transform.transform_path(path_component);
                    }
                }

                path
            } else {
                path
            };

            path
        })
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, when: Duration) -> Arc<VectorProperties> {
        let properties = self.brush.update_properties(properties, when);
        let properties = self.brush_properties.update_properties(properties, when);

        properties
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));

        if properties.transformations.len() > 0 {
            // Transform the path
            let mut path = properties.transformations[0].transform_path(&self.path);

            for transform in properties.transformations.iter().skip(1) {
                path = transform.transform_path(&path);
            }

            // Draw the transformed path
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
