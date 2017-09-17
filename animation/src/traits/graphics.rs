
///
/// Represents a point in 2D space
///
pub struct Point2D {
    pub x : f32,
    pub y : f32
}

///
/// A colour, in RGB format
///
pub struct RgbColor {
    pub r : u8,
    pub g : u8,
    pub b : u8
}

///
/// Commands that can be used to draw a layer
///
pub enum GraphicsCommand {
    /// Resets the current state of the graphics subsystem
    ResetState,

    /// Starts a new path
    BeginPath,

    /// Adds a new point to the current path. New paths must begin with a Move
    Move(Point2D),

    /// Adds a line to a particular point to the current path
    Line(Point2D),

    /// Adds a bezier curve to the current path. Parameters are the target point followed by the first and second control points
    Bezier(Point2D, Point2D, Point2D),

    /// Draws a line around the current path
    StrokePath,

    /// Fills the current path
    FillPath,

    /// Sets the fill colour
    SetFillRgb(RgbColor),

    /// Sets the line colour
    SetLineRgb(RgbColor),

    /// Sets the line width
    SetLineWidth(f32),
}

pub use GraphicsCommand::*;

