use flo_canvas::*;

use super::core_graphics_ffi::*;

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
    pub unsafe fn init(context: CGContextRef) -> CanvasLayer {
        CanvasLayer { 
            context: context
        }
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&self, draw: &Draw) {
        use self::Draw::*;

        unsafe {
            match draw {
                NewPath                                             => { CGContextBeginPath(self.context); }
                Move(x, y)                                          => { CGContextMoveToPoint(self.context, *x as CGFloat, *y as CGFloat); }
                Line(x, y)                                          => { CGContextAddLineToPoint(self.context, *x as CGFloat, *y as CGFloat); }
                BezierCurve((ex, ey), (c1x, c1y), (c2x, c2y))       => { CGContextAddCurveToPoint(self.context, *c1x as CGFloat, *c1y as CGFloat, *c2x as CGFloat, *c2y as CGFloat, *ex as CGFloat, *ey as CGFloat); }
                ClosePath                                           => { CGContextClosePath(self.context); }
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
}
