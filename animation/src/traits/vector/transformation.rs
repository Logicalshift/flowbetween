use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;

use crate::traits::edit::*;
use crate::traits::path::*;

use flo_curves::*;
use flo_canvas::*;

use smallvec::*;

use std::sync::*;
use std::time::{Duration};

///
/// Vector element that represents the possible transformations that can be
///
#[derive(Clone, PartialEq, Debug)]
pub enum Transformation {
    /// A 2D transformation matrix
    Matrix([[f64; 3]; 3]),

    /// A translation offset
    Translate(f64, f64),

    /// Flip horizontally around a particular point 
    FlipHoriz(f64, f64),

    /// Flip vertically around a particular point
    FlipVert(f64, f64),

    /// Scale around a point
    Scale(f64, f64, (f64, f64)),

    /// Rotate around a point
    Rotate(f64, (f64, f64))
}

impl Transformation {
    ///
    /// Converts an element transform to a transformation
    ///
    /// Not all element transforms can be converted by this routine (eg, this doesn't know bounding boxes so can't
    /// perform alignment transforms)
    ///
    pub fn from_element_transform<Coord>(origin: &Coord, transform: ElementTransform) -> Transformation
    where Coord: Coordinate+Coordinate2D {
        let (ox, oy) = (origin.x(), origin.y());

        match transform {
            ElementTransform::SetAnchor(_, _)   => Transformation::Translate(0.0, 0.0),
            ElementTransform::Align(_)          => Transformation::Translate(0.0, 0.0),
            ElementTransform::FlipHorizontal    => Transformation::FlipHoriz(ox, oy),
            ElementTransform::FlipVertical      => Transformation::FlipVert(ox, oy),
            ElementTransform::Scale(sx, sy)     => Transformation::Scale(sx, sy, (ox, oy)),
            ElementTransform::Rotate(theta)     => Transformation::Rotate(theta, (ox, oy)),
            ElementTransform::MoveTo(x, y)      => Transformation::Translate(x+ox, y+oy),
        }
    }

    ///
    /// Returns the inverse of this transformation
    ///
    pub fn invert(&self) -> Option<Transformation> {
        use self::Transformation::*;

        match self {
            Matrix(matrix)                  => Self::invert_matrix(matrix).map(|inverted_matrix| Matrix(inverted_matrix)),
            Translate(dx, dy)               => Some(Translate(-dx, -dy)),
            FlipHoriz(x, y)                 => Some(FlipHoriz(*x, *y)),
            FlipVert(x, y)                  => Some(FlipVert(*x, *y)),
            Scale(xratio, yratio, origin)   => Some(Scale(1.0/xratio, 1.0/yratio, *origin)),
            Rotate(angle, origin)           => Some(Rotate(-angle, *origin))
        }
    }

    ///
    /// Retrieves the matrix for this transformation
    ///
    fn get_matrix(&self) -> Option<[[f64; 3]; 3]> {
        use self::Transformation::*;

        match self {
            Matrix(matrix)                  => Some(*matrix),
            Translate(dx, dy)               => Some([[1.0, 0.0, *dx], [0.0, 1.0, *dy], [0.0, 0.0, 1.0]]),

            FlipHoriz(x, _y)                => Some([[-1.0, 0.0, 2.0*x], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
            FlipVert(_x, y)                 => Some([[1.0, 0.0, 0.0], [0.0, -1.0, 2.0*y], [0.0, 0.0, 1.0]]),

            Scale(xratio, yratio, (x, y))   => Some([[*xratio, 0.0, -xratio*x+x], [0.0, *yratio, -yratio*y+y], [0.0, 0.0, 1.0]]),
            Rotate(angle, (x, y))           => {
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                Some([[cos_a, -sin_a, -x*cos_a+y*sin_a+x], [sin_a, cos_a, -x*sin_a-y*cos_a+y], [0.0, 0.0, 1.0]])
            }
        }
    }

    ///
    /// Converts this transformation to a matrix transformation
    ///
    pub fn to_matrix(&self) -> Option<Transformation> {
        self.get_matrix()
            .map(|matrix| Transformation::Matrix(matrix))
    }

    ///
    /// Transforms a 2D point using this transformation
    ///
    pub fn transform_point<Coord>(&self, point: &Coord) -> Coord
    where Coord: Coordinate {
        // We translate the X and Y coordinates
        let num_components  = Coord::len();
        let (x, y)          = (point.get(0), point.get(1));

        let (x, y)          = match self {
            Transformation::Matrix(matrix)      => Self::transform_matrix(x, y, matrix),
            Transformation::Translate(dx, dy)   => (x + dx, y + dy),

            _other                              => Self::transform_matrix(x, y, &self.get_matrix().unwrap())
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
    /// Applies this transformation to a control point
    ///
    pub fn transform_control_point(&self, point: &ControlPoint) -> ControlPoint {
        match point {
            ControlPoint::BezierPoint(x, y)         => { let Coord2(x, y) = self.transform_point(&Coord2(*x as f64, *y as f64)); ControlPoint::BezierPoint(x as f32, y as f32) }
            ControlPoint::BezierControlPoint(x, y)  => { let Coord2(x, y) = self.transform_point(&Coord2(*x as f64, *y as f64)); ControlPoint::BezierControlPoint(x as f32, y as f32) }
        }
    }

    ///
    /// Transforms a bezier curve using this transformation
    ///
    pub fn transform_curve<Curve, NewCurve>(&self, curve: &Curve) -> NewCurve
    where   Curve:      BezierCurve,
            NewCurve:   BezierCurveFactory<Point=Curve::Point> {
        let start_point = self.transform_point(&curve.start_point());
        let end_point   = self.transform_point(&curve.end_point());
        let (cp1, cp2)  = curve.control_points();
        let (cp1, cp2)  = (self.transform_point(&cp1), self.transform_point(&cp2));

        NewCurve::from_points(start_point, (cp1, cp2), end_point)
    }

    ///
    /// Transforms a 2D point via the matrix transformation
    ///
    fn transform_matrix(x: f64, y: f64, matrix: &[[f64; 3]; 3]) -> (f64, f64) {
        let x1 = matrix[0][0] * x + matrix[0][1] * y + matrix[0][2];
        let y1 = matrix[1][0] * x + matrix[1][1] * y + matrix[1][2];

        (x1, y1)
    }

    ///
    /// Computes the determinant of a 2x2 matrix
    ///
    fn det2(matrix: &[[f64; 2]; 2]) -> f64 {
        matrix[0][0]*matrix[1][1] + matrix[0][1]*matrix[1][0]
    }

    ///
    /// Computes the minor of a 3x3 matrix
    ///
    fn minor3(matrix: &[[f64; 3]; 3], row: usize, col: usize) -> f64 {
        let (x1, x2)    = match row { 0 => (1, 2), 1 => (0, 2), 2 => (0, 1), _ => (0, 1) };
        let (y1, y2)    = match col { 0 => (1, 2), 1 => (0, 2), 2 => (0, 1), _ => (0, 1) };

        let matrix      = [
            [matrix[y1][x1], matrix[y1][x2]], 
            [matrix[y2][x1], matrix[y2][x2]]
        ];

        Self::det2(&matrix)
    }

    ///
    /// Computes the cofactor of an element in a 3x3 matrix
    ///
    fn cofactor3(matrix: &[[f64; 3]; 3], row: usize, col: usize) -> f64 {
        let minor   = Self::minor3(matrix, row, col);
        let sign    = (col&1) ^ (row&1);

        if sign != 0 {
            -minor 
        } else {
            minor
        }
    }

    ///
    /// Inverts a matrix transform
    ///
    fn invert_matrix(matrix: &[[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
        let cofactors   = [
            [Self::cofactor3(&matrix, 0, 0), Self::cofactor3(&matrix, 1, 0), Self::cofactor3(&matrix, 2, 0)],
            [Self::cofactor3(&matrix, 0, 1), Self::cofactor3(&matrix, 1, 1), Self::cofactor3(&matrix, 2, 1)],
            [Self::cofactor3(&matrix, 0, 2), Self::cofactor3(&matrix, 1, 2), Self::cofactor3(&matrix, 2, 2)],
        ];

        let det         = matrix[0][0]*cofactors[0][0] + matrix[0][1]*cofactors[0][1] + matrix[0][2]*cofactors[0][2];

        if det != 0.0 {
            let inv_det = 1.0/det;

            Some([
                [inv_det * cofactors[0][0], inv_det * cofactors[1][0], inv_det * cofactors[2][0]],
                [inv_det * cofactors[0][1], inv_det * cofactors[1][1], inv_det * cofactors[2][1]],
                [inv_det * cofactors[0][2], inv_det * cofactors[1][2], inv_det * cofactors[2][2]]
            ])
        } else {
            None
        }
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

impl VectorElement for (ElementId, SmallVec<[Transformation; 2]>) {
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
        // Clone the properties
        let mut new_properties          = (*properties).clone();
        let mut new_transformations     = (*properties.transformations).clone();

        // Apply the transformations from this item to the new properties
        new_transformations.extend(self.1.iter().cloned());
        new_properties.transformations  = Arc::new(new_transformations);

        // Result is the updated properties
        Arc::new(new_properties)
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, _properties: &VectorProperties) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        Vector::Transformation(self.clone())
    }
}

impl Into<Transform2D> for Transformation {
    fn into(self) -> Transform2D {
        let matrix = self.get_matrix().unwrap_or_else(|| [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

        Transform2D([
            [matrix[0][0] as f32, matrix[0][1] as f32, matrix[0][2] as f32], 
            [matrix[1][0] as f32, matrix[1][1] as f32, matrix[1][2] as f32],
            [matrix[2][0] as f32, matrix[2][1] as f32, matrix[2][2] as f32]
        ])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::f64;

    #[test]
    fn translate_point() {
        let transform           = Transformation::Translate(1.0, 2.0);
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 43.0).abs() < 0.001);
        assert!((translated_point.y() - 47.0).abs() < 0.001);
    }

    #[test]
    fn translate_point_matrix() {
        let transform           = Transformation::Translate(1.0, 2.0).to_matrix().unwrap();
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 43.0).abs() < 0.001);
        assert!((translated_point.y() - 47.0).abs() < 0.001);
    }

    #[test]
    fn flip_horiz_around_origin() {
        let transform           = Transformation::FlipHoriz(0.0, 0.0);
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - -42.0).abs() < 0.001);
        assert!((translated_point.y() - 45.0).abs() < 0.001);
    }

    #[test]
    fn flip_horiz_around_offset() {
        let transform           = Transformation::FlipHoriz(41.0, 44.0);
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);
        println!("{:?}", translated_point);

        assert!((translated_point.x() - 40.0).abs() < 0.001);
        assert!((translated_point.y() - 45.0).abs() < 0.001);
    }

    #[test]
    fn flip_vertically_around_origin() {
        let transform           = Transformation::FlipVert(0.0, 0.0);
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 42.0).abs() < 0.001);
        assert!((translated_point.y() - -45.0).abs() < 0.001);
    }

    #[test]
    fn flip_vertically_around_offset() {
        let transform           = Transformation::FlipVert(41.0, 44.0);
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 42.0).abs() < 0.001);
        assert!((translated_point.y() - 43.0).abs() < 0.001);
    }

    #[test]
    fn scale_about_center() {
        let transform           = Transformation::Scale(2.0, 3.0, (0.0, 0.0));
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 84.0).abs() < 0.001);
        assert!((translated_point.y() - 135.0).abs() < 0.001);
    }

    #[test]
    fn scale_about_point() {
        let transform           = Transformation::Scale(2.0, 3.0, (43.0, 45.0));
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 41.0).abs() < 0.001);
        assert!((translated_point.y() - 45.0).abs() < 0.001);
    }

    #[test]
    fn rotate_about_center() {
        let transform           = Transformation::Rotate(f64::consts::PI/2.0, (0.0, 0.0));
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - -45.0).abs() < 0.001);
        assert!((translated_point.y() - 42.0).abs() < 0.001);
    }

    #[test]
    fn rotate_abount_point() {
        let transform           = Transformation::Rotate(f64::consts::PI/2.0, (41.0, 44.0));
        let source_point        = Coord2(42.0, 45.0);

        let translated_point    = transform.transform_point(&source_point);

        assert!((translated_point.x() - 40.0).abs() < 0.001);
        assert!((translated_point.y() - 45.0).abs() < 0.001);
    }
}
