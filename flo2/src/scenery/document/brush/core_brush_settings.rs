use crate::scenery::document::canvas::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use serde::*;

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

        let brush_daub_settings = BrushDaubSettings {
            shape:          path,
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
    pub shape: CanvasPath,

    /// The base radius of the shape (used for varying the size of the daub)
    pub base_radius: f64,

    /// The distance between daubs (applied irrespective of scale). 0.5 is a good value for a brush that's suposed to create a smooth stroke
    pub distance: f64,

    /// The minimum error allowed in the fit for this brush (>1.0 is a good value for a smooth brush stroke)
    pub fit: f64,
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