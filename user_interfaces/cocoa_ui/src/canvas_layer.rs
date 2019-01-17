use flo_canvas::*;

use super::core_graphics_ffi::*;

///
/// Processes canvas draw commands onto a core graphics context
/// 
/// This assumes that all the commands are intended for a specific layer: ie, layer switch commands
/// are ignored.
///
pub struct CanvasLayer {
    /// The CGContext that drawing commands for this layer should be sent to
    context: CFRef<CGContextRef>,

    /// The SRGB colour space
    srgb: CFRef<CGColorSpaceRef>
}

impl CanvasLayer {
    ///
    /// Creates a new canvas layer that will render to the specified context
    ///
    pub unsafe fn new(context: CGContextRef) -> CanvasLayer {
        // Colours are in the SRGB colourspace
        let srgb = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);

        // We take ownership of the context (it should be retained already)
        CanvasLayer { 
            context:    CFRef::from(context),
            srgb:       CFRef::from(srgb)
        }
    }

    ///
    /// Creates a CGColorRef from a canvas colour
    ///
    #[inline] fn create_color_ref(&self, color: &Color) -> CFRef<CGColorRef> {
        unsafe {
            let (r, g, b, a)    = color.to_rgba_components();
            let components      = [r as CGFloat, g as CGFloat, b as CGFloat, a as CGFloat];
            let color           = CGColorCreate(*self.srgb, components.as_ptr());

            CFRef::from(color)
        }
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&self, draw: &Draw) {
        use self::Draw::*;

        unsafe {
            match draw {
                NewPath                                             => { CGContextBeginPath(*self.context); }
                Move(x, y)                                          => { CGContextMoveToPoint(*self.context, *x as CGFloat, *y as CGFloat); }
                Line(x, y)                                          => { CGContextAddLineToPoint(*self.context, *x as CGFloat, *y as CGFloat); }
                BezierCurve((ex, ey), (c1x, c1y), (c2x, c2y))       => { CGContextAddCurveToPoint(*self.context, *c1x as CGFloat, *c1y as CGFloat, *c2x as CGFloat, *c2y as CGFloat, *ex as CGFloat, *ey as CGFloat); }
                ClosePath                                           => { CGContextClosePath(*self.context); }
                Fill                                                => { CGContextFillPath(*self.context); }
                Stroke                                              => { CGContextStrokePath(*self.context); }
                LineWidth(width)                                    => { CGContextSetLineWidth(*self.context, *width as CGFloat); }
                LineWidthPixels(width_pixels)                       => { /* TODO */ }
                LineJoin(join)                                      => { /* TODO */ }
                LineCap(cap)                                        => { /* TODO */ }
                NewDashPattern                                      => { /* TODO */ }
                DashLength(len)                                     => { /* TODO */ }
                DashOffset(offset)                                  => { /* TODO */ }
                FillColor(col)                                      => { CGContextSetFillColorWithColor(*self.context, *self.create_color_ref(col)); }
                StrokeColor(col)                                    => { CGContextSetStrokeColorWithColor(*self.context, *self.create_color_ref(col)); }
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
