use super::brush_response::*;
use super::shape_streams::*;
use crate::scenery::document::canvas::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_curves::bezier::rasterize::*;
use flo_curves::bezier::vectorize::*;

use futures::prelude::*;
use serde::*;

use std::sync::*;

///
/// The core brush settings describe how a brush stroke is turned into a shape
///
/// This misses out things like 
///
#[derive(Serialize, Deserialize)]
pub struct CoreBrushSettings {
    /// How the shape should be built
    pub builder: BrushShapeBuilder,

    /// What the pressure will vary (or the empty vec if the pressure has no effect)
    pub pressure_vary: Vec<BrushVary>,

    /// What the stroke speed will vary (or the empty vec if the speed has no effect)
    pub speed_vary: Vec<BrushVary>,
}

impl CoreBrushSettings {
    ///
    /// Creates the default daub brush from a path
    ///
    pub fn with_path(path: impl Into<CanvasPath>) -> Self {
        // Convert the path
        let path            = path.into();
        let working_path    = WorkingSubpath::from_canvas_path(&path);

        // Get the size of the path
        let bounds = working_path.iter()
            .map(|subpath| path_bounding_box::<_, Bounds<_>>(subpath))
            .reduce(|b1, b2| b1.union_bounds(b2));
        let bounds = bounds.unwrap_or(Bounds::empty());

        // Radius comes from the bounds
        let width       = bounds.max().x - bounds.min().x;
        let height      = bounds.max().y - bounds.min().y;
        let diameter    = width.max(height);
        let radius      = diameter/2.0;

        // Transform the path so that bounds.min() maps to 0,0
        let working_path = working_path.iter()
            .map(|subpath| subpath.map_points(|mut point| {
                point.x -= bounds.min().x;
                point.y -= bounds.min().y;

                point
            }))
            .collect::<Vec<_>>();

        let brush_daub_settings = BrushDaubSettings {
            shape:          working_path,
            bounds:         (bounds.min(), bounds.max()),
            base_radius:    radius,
            distance:       0.5,
            fit:            1.0,
        };

        // Create the brush settings for a 'standard' pressure sensitive brush
        CoreBrushSettings { 
            builder:        BrushShapeBuilder::Daubs(brush_daub_settings), 
            pressure_vary:  vec![BrushVary::Radius { min: 0.0, max: 1.0, profile: vec![ResponseCurve::linear()] }], 
            speed_vary:     vec![],
        }
    }

    ///
    /// Creates the simple line-width brush
    ///
    pub fn line_width_brush() -> Self {
        CoreBrushSettings { 
            builder:        BrushShapeBuilder::LineWidth, 
            pressure_vary:  vec![BrushVary::Radius { min: 0.0, max: 1.0, profile: vec![ResponseCurve::linear()] }], 
            speed_vary:     vec![],
        }
    }

    ///
    /// Creates the brush responses that describe how to generate this part of the brush
    ///
    pub fn to_brush_responses(&self) -> Vec<BrushResponse> {
        let create_shape = match &self.builder {
            BrushShapeBuilder::Daubs(daubs) => { daubs.create_shape_response() }
            BrushShapeBuilder::LineWidth    => { todo!() }
        };

        vec![create_shape]
    }
}

impl Default for CoreBrushSettings {
    fn default() -> Self {
        use flo_curves::arc::*;

        let base_path = Circle::new(WorkingPoint { x: 0.0, y: 0.0 }, 10.0);
        let base_path = base_path.to_path::<WorkingSubpath>();
        let base_path = WorkingSubpath::to_canvas_path(&[base_path]);

        Self::with_path(base_path)
    }
}

///
/// How the shape of the brush is built up
///
#[derive(Serialize, Deserialize)]
pub enum BrushShapeBuilder {
    /// Simple 'line width' brush
    LineWidth,

    /// Build up the brush using 'daubs' (shapes repeatedly stamped and converted into a vector path)
    Daubs(BrushDaubSettings),
}

///
/// Settings for generating a brush daub
///
#[derive(Serialize, Deserialize)]
pub struct BrushDaubSettings {
    /// The path that makes up the daub shape. This should be a path centered around the 0,0 point (the 0,0 point is where this shape will be scaled around)
    pub shape: Vec<WorkingSubpath>,

    /// The size of the shape, as a minimum and maximum value
    pub bounds: (WorkingPoint, WorkingPoint),

    /// The base radius of the shape (used for varying the size of the daub)
    pub base_radius: f64,

    /// The distance between daubs (applied irrespective of scale). 0.5 is a good value for a brush that's suposed to create a smooth stroke
    pub distance: f64,

    /// The minimum error allowed in the fit for this brush (>1.0 is a good value for a smooth brush stroke)
    pub fit: f64,
}

impl BrushDaubSettings {
    ///
    /// Creates the 'create brush shape' program for these daub settings
    ///
    pub fn create_shape_response(&self) -> BrushResponse {
        let daub_size       = ContourSize((self.bounds.1.x - self.bounds.0.x).ceil() as usize, (self.bounds.1.y - self.bounds.0.y).ceil() as usize);
        let daub_contour    = Arc::new(PathContour::from_path(self.shape.clone(), daub_size));
        let max_error       = self.fit;

        BrushResponse::ShapeGenerator(Arc::new(move |points| {
            daub_brush_stream(daub_contour.clone(), points, max_error).boxed()
        }))
    }
}

///
/// Describes what an input parameter should vary
///
#[derive(Serialize, Deserialize)]
pub enum BrushVary {
    /// Change the radius of the brush stroke based on this parameter, between the minimum and maximum values (with the specified response curve)
    Radius { min: f64, max: f64, profile: Vec<ResponseCurve>, },

    /// Change the distance between daubs based on this parameter
    Distance { min: f64, max: f64, profile: Vec<ResponseCurve>, },
}

///
/// 1D Bezier curve that describes how an input value should map to an output value
///
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ResponseCurve(pub [f64; 4]);

impl Geo for ResponseCurve {
    type Point = f64;
}

impl BezierCurve for ResponseCurve {
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.0[0]
    }

    #[inline]
    fn end_point(&self) -> Self::Point {
        self.0[3]
    }

    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.0[1], self.0[2])
    }
}

impl ResponseCurve {
    ///
    /// Creates a linear response curve
    ///
    pub fn linear() -> Self {
        ResponseCurve([0.0, 1.0/3.0, 2.0/3.0, 1.0])
    }
}