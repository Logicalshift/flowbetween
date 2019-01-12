use flo_canvas::*;

use core_graphics::context::*;

///
/// Processes canvas draw commands onto a core graphics context
/// 
/// This assumes that all the commands are intended for a specific layer: ie, layer switch commands
/// are ignored.
///
pub struct CanvasLayer {
    context: CGContextRef
}

impl CanvasLayer {
    ///
    /// Creates a new canvas layer that will render to the specified context
    ///
    pub fn init(context: CGContextRef) -> CanvasLayer {
        CanvasLayer { 
            context: context
        }
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&self, draw: &Draw) {
        use self::Draw::*;

        match draw {
            NewPath                                             => { /* TODO */ }
            Move(x, y)                                          => { /* TODO */ }
            Line(x, y)                                          => { /* TODO */ }
            BezierCurve((ex, ey), (c1x, c1y), (c2x, c2y))       => { /* TODO */ }
            ClosePath                                           => { /* TODO */ }
            Fill                                                => { /* TODO */ }
            Stroke                                              => { /* TODO */ }
            LineWidth(width)                                    => { /* TODO */ }
            LineWidthPixels(width_pixels)                       => { /* TODO */ }
            LineJoin(join)                                      => { /* TODO */ }
            LineCap(cap)                                        => { /* TODO */ }
            NewDashPattern                                      => { /* TODO */ }
            DashLength(len)                                     => { /* TODO */ }
            DashOffset(offset)                                  => { /* TODO */ }
            FillColor(col)                                      => { /* TODO */ }
            StrokeColor(col)                                    => { /* TODO */ }
            BlendMode(blend)                                    => { /* TODO */ }
            IdentityTransform                                   => { /* TODO */ }
            CanvasHeight(height)                                => { /* TODO */ }
            CenterRegion((minx, miny), (maxx, maxy))            => { /* TODO */ }
            MultiplyTransform(transform)                        => { /* TODO */ }
            Unclip                                              => { /* TODO */ }
            Clip                                                => { /* TODO */ }
            Store                                               => { /* TODO */ }
            Restore                                             => { /* TODO */ }
            FreeStoredBuffer                                    => { /* TODO */ }
            PushState                                           => { /* TODO */ }
            PopState                                            => { /* TODO */ }
            ClearCanvas                                         => { /* TODO */ }
            Layer(layer_id)                                     => { /* TODO */ }
            LayerBlend(layer_id, blend)                         => { /* TODO */ }
            ClearLayer                                          => { /* TODO */ }
        }
    }
}
