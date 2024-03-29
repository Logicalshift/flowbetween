use crate::path::animation_path_attributes::*;

use flo_canvas::*;

use std::sync::*;
use std::time::{Duration};

///
/// The state of a layer as captured by the routines in `drawing_to_path`
///
#[derive(Clone, Debug)]
pub struct LayerState {
    /// The current time that rendering is occurring
    pub current_time:   Duration,

    /// The path that is currently defined for this layer
    pub current_path:   Vec<PathOp>,

    /// The most recently cached path (so we re-use the same vec if multiple animation paths are generated from one set of operations)
    pub cached_path:    Option<Arc<Vec<PathOp>>>,

    /// The current stroke state
    pub stroke:         StrokeState,

    /// The current fill state
    pub fill:           FillState,

    /// The selected blend mode
    pub blend_mode:     BlendMode,

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
            current_time:   Duration::from_millis(0),
            current_path:   vec![],
            cached_path:    None,
            stroke:         StrokeState::default(),
            fill:           FillState::default(),
            blend_mode:     BlendMode::SourceOver,
            transform:      None
        }
    }
}

impl FillState {
    pub fn into_attributes(&self, blend_mode: BlendMode) -> AnimationPathAttribute {
        match self.color {
            FillStyle::Solid(color)                                 => AnimationPathAttribute::Fill(blend_mode, color, self.winding_rule),
            FillStyle::Texture(texture_id, (x1, y1), (x2, y2))      => AnimationPathAttribute::FillTexture(blend_mode, texture_id, (x1, y1), (x2, y2), self.transform, self.winding_rule),
            FillStyle::Gradient(gradient_id, (x1, y1), (x2, y2))    => AnimationPathAttribute::FillGradient(blend_mode, gradient_id, (x1, y1), (x2, y2), self.transform, self.winding_rule),
        }
    }
}

impl StrokeState {
    pub fn into_attributes(&self, blend_mode: BlendMode) -> AnimationPathAttribute {
        match self.width {
            StrokeWidth::CanvasCoords(width)    => AnimationPathAttribute::Stroke(blend_mode, width, self.color, self.line_join, self.line_cap),
            StrokeWidth::PixelCoords(width)     => AnimationPathAttribute::StrokePixels(blend_mode, width, self.color, self.line_join, self.line_cap)
        }
    }
}
