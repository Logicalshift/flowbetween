use super::super::traits::*;

use flo_curves::*;
use itertools::*;

use std::iter;
use std::sync::*;

///
/// The type of a particular vector edge (defines how it interacts with the ray)
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RaycastEdgeKind {
    // Solid edge
    Solid,

    // Edge that hides whatever is beneath it
    EraseContents
}

///
/// Represents a raycasting edge
///
pub struct RaycastEdge {
    /// The curve representing this particular edge
    pub curve: PathCurve,

    /// The type of this edge
    pub kind: RaycastEdgeKind,

    /// The element ID that this edge was from
    pub element_id: ElementId
}

impl RaycastEdge {
    ///
    /// Retrieves the edges corresponding to a particular vector object (when drawn with the specified vector properties)
    ///
    pub fn from_vector<'a>(vector: &'a Vector, properties: Arc<VectorProperties>) -> Box<dyn 'a+Iterator<Item=Self>> {
        match vector {
            Vector::BrushDefinition(_defn)      => { Box::new(iter::empty()) }
            Vector::BrushProperties(_props)     => { Box::new(iter::empty()) }
            Vector::Motion(_motion)             => { Box::new(iter::empty()) }
            Vector::Transformation(_transform)  => { Box::new(iter::empty()) }
            Vector::Error                       => { Box::new(iter::empty()) }

            Vector::Transformed(transform)      => { Self::from_transformed(transform, properties) }
            Vector::BrushStroke(brush_stroke)   => { Self::from_brush_stroke(brush_stroke, properties) }
            Vector::Path(path)                  => { Box::new(Self::from_path_element(path)) }
            Vector::Group(group_element)        => { Box::new(Self::from_group(group_element, properties)) }
        }
    }

    ///
    /// Retrieves the edges corresponding to a transformed element
    ///
    pub fn from_transformed<'a>(vector: &'a TransformedVector, properties: Arc<VectorProperties>) -> Box<dyn 'a+Iterator<Item=Self>> {
        // The transformed vector here is an Arc, so we can borrow it for long enough
        // But this requires both the reference and its borrow to live in the same place, and Rust does not support that
        // So transformed vectors will run slowly as we have to store them in a temporary Vec to get around this
        let transformed_vector  = vector.transformed_vector();
        let edges               = Self::from_vector(&*transformed_vector, properties);
        let edge_collection     = edges.collect::<Vec<_>>();

        Box::new(edge_collection.into_iter())
    }

    ///
    /// Retrieves the edges corresponding to a group element
    ///
    pub fn from_group<'a>(group: &'a GroupElement, properties: Arc<VectorProperties>) -> impl 'a+Iterator<Item=Self> {
        let element_id = group.id();

        group.elements()
            .flat_map(move |element| Self::from_vector(element, properties.clone()))
            .map(move |mut element| {
                element.element_id = element_id;
                element
            })
    }

    ///
    /// Retrieves the edges corresponding to a path element
    ///
    pub fn from_path_element<'a>(vector: &'a PathElement) -> impl 'a+Iterator<Item=Self> {
        match vector.brush().drawing_style() {
            BrushDrawingStyle::Erase    => { Self::from_path(vector.id(), vector.path(), RaycastEdgeKind::EraseContents) }
            BrushDrawingStyle::Draw     => { Self::from_path(vector.id(), vector.path(), RaycastEdgeKind::Solid) }
        }
    }

    ///
    /// Returns a particular path as ray cast edges
    ///
    pub fn from_path<'a>(element_id: ElementId, path: &'a Path, edge_kind: RaycastEdgeKind) -> impl 'a+Iterator<Item=Self> {
        path.to_curves()
            .map(move |curve| {
                Self {
                    curve:          curve,
                    kind:           edge_kind,
                    element_id:     element_id
                }
            })
    }

    ///
    /// Returns the edges from a brush stroke element
    ///
    pub fn from_brush_stroke<'a>(brush_stroke: &'a BrushElement, properties: Arc<VectorProperties>) -> Box<dyn 'a+Iterator<Item=Self>> {
        match properties.brush.drawing_style() {
            BrushDrawingStyle::Erase    => {
                let element_id = brush_stroke.id();

                // Ignore any elements underneath the entire path for an erasing brush stroke
                Box::new(brush_stroke.to_path(&*properties, PathConversion::Fastest).unwrap()
                    .into_iter()
                    .flat_map(move |path| Self::from_path(element_id, &path, RaycastEdgeKind::EraseContents).collect::<Vec<_>>()))
            }

            BrushDrawingStyle::Draw     => {
                // A draw brush stroke just adds a single edge
                let points  = brush_stroke.points();
                let paths   = Self::from_brush_points(brush_stroke.id(), &*points, RaycastEdgeKind::Solid);
                let paths   = paths.collect::<Vec<_>>();
                Box::new(paths.into_iter())
            }
        }
    }

    ///
    /// Converts a position from a `BrushPoint` to a `PathPoint`
    ///
    #[inline] fn brush_to_path_point(brush_point: (f32, f32)) -> PathPoint {
        PathPoint { position: (brush_point.0 as f64, brush_point.1 as f64) }
    }

    ///
    /// Converts a set of brush points into a set of
    ///
    pub fn from_brush_points<'a, PointIter: 'a+IntoIterator<Item=&'a BrushPoint>>(element_id: ElementId, points: PointIter, edge_kind: RaycastEdgeKind) -> Box<dyn 'a+Iterator<Item=RaycastEdge>> {
        Box::new(points.into_iter()
            .tuple_windows()
            .map(|(prev, next)| {
                let start_point = Self::brush_to_path_point(prev.position);
                let cp1         = Self::brush_to_path_point(next.cp1);
                let cp2         = Self::brush_to_path_point(next.cp2);
                let end_point   = Self::brush_to_path_point(next.position);

                PathCurve::from_points(start_point, (cp1, cp2), end_point)
            })
            .map(move |curve| Self {
                curve:      curve,
                kind:       edge_kind,
                element_id: element_id
            }))
    }
}
