use super::path::*;
use super::paint::*;

use flo_canvas::*;
use nanovg::*;
use nanovg;

///
/// Represents state associated with sending canvas drawing commands to a nanovg frame
/// 
pub struct NanoVgDrawingState<'a> {
    /// Pending path instructions
    path: Vec<NanoVgPath>,

    /// Current stroke paint option
    stroke: NanoVgPaint,

    /// Current fill paint option
    fill: NanoVgPaint,

    /// The frame that we'll draw upon
    frame: &'a Frame<'a>
}

impl<'a> NanoVgDrawingState<'a> {
    ///
    /// Creates a new NanoVgDrawing state
    /// 
    pub fn new(frame: &'a Frame<'a>) -> NanoVgDrawingState<'a> {
        NanoVgDrawingState {
            path:   vec![],
            stroke: NanoVgPaint::Color(nanovg::Color::new(0.0, 0.0, 0.0, 1.0)),
            fill:   NanoVgPaint::Color(nanovg::Color::new(0.0, 0.0, 0.0, 1.0)),
            frame:  frame
        }
    }

    pub fn draw(&mut self, drawing: Draw) {
        use self::Draw::*;

        match drawing {
            NewPath                                     => {},
            Move(x, y)                                  => {},
            Line(x, y)                                  => {},
            BezierCurve(pos, cp1, cp2)                  => {},
            ClosePath                                   => {},
            Fill                                        => {},
            Stroke                                      => {},
            LineWidth(width)                            => {},
            LineWidthPixels(width)                      => {},
            LineJoin(join)                              => {},
            LineCap(cap)                                => {},
            NewDashPattern                              => {},
            DashLength(len)                             => {},
            DashOffset(offset)                          => {},
            FillColor(col)                              => {},
            StrokeColor(col)                            => {},
            BlendMode(blend)                            => {},
            IdentityTransform                           => {},
            CanvasHeight(height)                        => {},
            CenterRegion((minx, miny), (maxx, maxy))    => {},
            MultiplyTransform(transform)                => {},
            Unclip                                      => {},
            Clip                                        => {},
            Store                                       => {},
            Restore                                     => {},
            FreeStoredBuffer                            => {},
            PushState                                   => {},
            PopState                                    => {},
            ClearCanvas                                 => {},
            Layer(layer_id)                             => {},
            LayerBlend(layer_id, mode)                  => {},
            ClearLayer                                  => {}
        }
    }
}
