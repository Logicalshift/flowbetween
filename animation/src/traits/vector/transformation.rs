use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;

use crate::traits::edit::*;
use crate::traits::path::*;
use crate::traits::motion::*;

use flo_curves::*;
use flo_canvas::*;

use std::sync::*;
use std::time::{Duration};

///
/// Vector element that represents the possible transformations that can be
///
#[derive(Clone, PartialEq, Debug)]
pub enum Transformation {
    /// A 2D transformation matrix
    Matrix([[f64; 3]; 3])
}

impl Transformation {
    ///
    /// Transforms a 2D point using this transformation
    ///
    pub fn transform_point<Coord>(&self, point: &Coord) -> Coord
    where Coord: Coordinate {
        // We translate the X and Y coordinates
        let num_components  = Coord::len();
        let (x, y)          = (point.get(0), point.get(1));

        let (x, y)          = match self {
            Transformation::Matrix(matrix)  => Self::transform_matrix(x, y, matrix)
        };

        // The rest of the points are let through as-is
        let mut components  = vec![0.0; num_components];
        components[0]       = x;
        components[1]       = y;

        for index in 2..num_components {
            components[index] = point.get(index);
        }

        // Build the final result
        Coord::from_components(&components)
    }

    ///
    /// Transforms a 2D point via the matrix transformation
    ///
    fn transform_matrix(x: f64, y: f64, matrix: &[[f64; 3]; 3]) -> (f64, f64) {
        let x = matrix[0][0] * x + matrix[0][1] * y + matrix[0][2];
        let y = matrix[1][0] * x + matrix[1][1] * y + matrix[1][2];

        (x, y)
    }

    ///
    /// Applies this transformation to a path point
    ///
    pub fn transform_path_point(&self, point: &PathPoint) -> PathPoint {
        let new_position = self.transform_point(&Coord2(point.position.0, point.position.1));

        PathPoint {
            position: (new_position.x(), new_position.y())
        }
    }

    ///
    /// Transforms a path component via this transformation
    ///
    pub fn transform_path_component(&self, component: &PathComponent) -> PathComponent {
        use self::PathComponent::*;

        match component {
            Move(point)         => Move(self.transform_path_point(point)),
            Line(point)         => Line(self.transform_path_point(point)),
            Bezier(p1, p2, p3)  => Bezier(self.transform_path_point(p1), self.transform_path_point(p2), self.transform_path_point(p3)),

            Close               => Close,
        }
    }

    ///
    /// Transforms a path via this transformation
    ///
    pub fn transform_path(&self, path: &Path) -> Path {
        let mut new_elements = vec![];

        // Transform each of the components
        for component in path.elements.iter() {
            new_elements.push(self.transform_path_component(component));
        }

        // Build into a new path
        Path::from_elements(new_elements)
    }
}

impl VectorElement for (ElementId, Transformation) {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId {
        self.0
    }

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.0 = new_id;
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, _gc: &mut dyn GraphicsPrimitives, _properties: &VectorProperties, _when: Duration) {

    }

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { 
        properties
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, _motion: &Motion, _when: Duration) -> Vector {
        Vector::Transformation(self.clone())
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::Transformation(self.clone())
    }
}
