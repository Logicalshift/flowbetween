use flo_canvas::*;

///
/// The state of a layer as captured by the routines in `drawing_to_path`
///
#[derive(Clone, Debug)]
pub struct LayerState {
    /// The path that is currently defined for this layer
    pub current_path:   Vec<PathOp>,

    /// The current stroke state
    pub stroke:         StrokeState,

    /// The current fill state
    pub fill:           FillState,

    /// If a transform multiplication has been applied, this is the transformation
    pub transform:      Option<Transform2D>
}

///
/// How the width of a stroke is defined
///
#[derive(Clone, Copy, Debug)]
pub enum StrokeWidth {
    CanvasCoords(f32),
    PixelCoords(f32)
}

///
/// The current settings for the next 'stroke' operation
///
#[derive(Clone, Debug)]
pub struct StrokeState {
    pub color:          Color,
    pub width:          StrokeWidth,
    pub line_join:      LineJoin,
    pub line_cap:       LineCap,
    pub dash_pattern:   Option<(f32, Vec<f32>)>
}

#[derive(Clone, Copy, Debug)]
pub enum FillStyle {
    Solid(Color),
    Texture(TextureId, (f32, f32), (f32, f32)),
    Gradient(GradientId, (f32, f32), (f32, f32))
}

///
/// The current settings for the next 'fill operation'
///
#[derive(Clone, Debug)]
pub struct FillState {
    pub color:          FillStyle,
    pub winding_rule:   WindingRule,
    pub transform:      Option<Transform2D>
}

impl Default for FillStyle {
    fn default() -> FillStyle { FillStyle::Solid(Color::Rgba(0.0, 0.0, 0.0, 1.0)) }
}

impl Default for FillState {
    fn default() -> FillState { 
        FillState {
            color:          FillStyle::default(),
            winding_rule:   WindingRule::EvenOdd,
            transform:      None
        }
    }
}

impl Default for StrokeState {
    fn default() -> StrokeState {
        StrokeState {
            color:          Color::Rgba(0.0, 0.0, 0.0, 1.0),
            width:          StrokeWidth::CanvasCoords(1.0),
            line_join:      LineJoin::Round,
            line_cap:       LineCap::Butt,
            dash_pattern:   None
        }
    }
}

impl Default for LayerState {
    fn default() -> LayerState {
        LayerState {
            current_path:   vec![],
            stroke:         StrokeState::default(),
            fill:           FillState::default(),
            transform:      None
        }
    }
}
