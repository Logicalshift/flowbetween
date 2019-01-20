use flo_canvas::*;

use super::canvas_state::*;
use super::core_graphics_ffi::*;

///
/// Processes canvas draw commands onto a core graphics context
/// 
/// This assumes that all the commands are intended for a specific layer: ie, layer switch commands
/// are ignored.
///
pub struct CanvasLayer {
    /// The location of the viewport origin for this canvas layer (the point that we should consider as 0,0)
    viewport_origin: (f64, f64),

    /// The width and height of the viewport for this layer
    viewport_size: (f64, f64),

    /// The width and height of the canvas for this layer (canvas is assumed to have an origin at 0,0)
    canvas_size: (f64, f64),

    /// Tracks the current state of the context
    state: CanvasState,

    /// The CGContext that drawing commands for this layer should be sent to
    context: CFRef<CGContextRef>
}

impl CanvasLayer {
    ///
    /// Creates a new canvas layer that will render to the specified context
    ///
    pub unsafe fn new(context: CFRef<CGContextRef>, viewport_origin: (f64, f64), viewport_size: (f64, f64), canvas_size: (f64, f64)) -> CanvasLayer {
        // Colours are in the SRGB colourspace
        let srgb        = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        let mut state   = CanvasState::new(CFRef::from(srgb));

        state.activate_context(context.clone());

        let mut new_layer = CanvasLayer {
            viewport_origin:    viewport_origin,
            viewport_size:      viewport_size,
            canvas_size:        canvas_size,
            context:            context,
            state:              state
        };

        new_layer.state.set_transform(new_layer.get_identity_transform());

        new_layer
    }

    ///
    /// Computes the identity transform for this canvas
    ///
    fn get_identity_transform(&self) -> CGAffineTransform {
        unsafe {
            let (origin_x, origin_y)    = self.viewport_origin;
            let (width, height)         = self.canvas_size;
            let scale                   = (height as CGFloat)/2.0;

            let transform = CGAffineTransformIdentity;
            let transform = CGAffineTransformTranslate(transform, origin_x as CGFloat, origin_y as CGFloat);
            let transform = CGAffineTransformTranslate(transform, (width as CGFloat)/2.0, (height as CGFloat)/2.0);
            let transform = CGAffineTransformScale(transform, scale, -scale);

            transform
        }
    }

    ///
    /// Computes a matrix to be appended to the identity transform that will set the height of the canvas
    ///
    fn get_height_transform(&self, height: f64) -> CGAffineTransform {
        unsafe {
            let mut ratio_x = 2.0/height;
            let ratio_y     = ratio_x;

            if height < 0.0 {
                ratio_x = -ratio_x;
            }

            let result = CGAffineTransformIdentity;
            let result = CGAffineTransformScale(result, ratio_x as CGFloat, ratio_y as CGFloat);

            result
        }
    }

    ///
    /// Retrieves the transformation needed to move the center of the canvas to the specified point
    ///
    pub fn get_center_transform(&self, minx: f64, miny: f64, maxx: f64, maxy: f64) -> CGAffineTransform {
        unsafe {
            let (origin_x, origin_y)        = self.viewport_origin;
            let (pixel_width, pixel_height) = self.canvas_size;
            let current_transform           = self.state.current_transform();

            // Get the current scaling of this canvas
            let mut xscale = (current_transform.a*current_transform.a + current_transform.b*current_transform.b).sqrt();
            let mut yscale = (current_transform.c*current_transform.c + current_transform.d*current_transform.d).sqrt();
            if xscale == 0.0 { xscale = 1.0; }
            if yscale == 0.0 { yscale = 1.0; }

            // Current X, Y coordinates (centered)
            let cur_x = (current_transform.tx-(pixel_width/2.0))/xscale;
            let cur_y = (current_transform.ty-(pixel_height/2.0))/yscale;
            
            // New center coordinates
            let center_x = (minx+maxx)/2.0;
            let center_y = (miny+maxy)/2.0;

            // Compute the offsets and transform the canvas
            let x_offset = cur_x - center_x;
            let y_offset = cur_y - center_y;

            let x_offset = x_offset + origin_x/xscale;
            let y_offset = y_offset + origin_y/xscale;

            // Generate the result matrix
            let result = CGAffineTransformIdentity;
            let result = CGAffineTransformTranslate(result, x_offset as CGFloat, y_offset as CGFloat);
            result
        }
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&mut self, draw: &Draw) {
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
                FillColor(col)                                      => { self.state.set_fill_color(col); }
                StrokeColor(col)                                    => { self.state.set_stroke_color(col); }
                BlendMode(blend)                                    => { /* TODO */ }
                Unclip                                              => { /* TODO */ }
                Clip                                                => { /* TODO */ }
                Store                                               => { /* TODO */ }
                Restore                                             => { /* TODO */ }
                FreeStoredBuffer                                    => { /* TODO */ }
                PushState                                           => { self.state.push_state(); }
                PopState                                            => { self.state.pop_state(); }
                ClearCanvas                                         => { /* TODO */ }
                Layer(layer_id)                                     => { /* TODO */ }
                LayerBlend(layer_id, blend)                         => { /* TODO */ }
                ClearLayer                                          => { /* TODO */ }

                IdentityTransform                                   => { 
                    self.state.set_transform(self.get_identity_transform());
                }
                CanvasHeight(height)                                => {
                    let identity    = self.get_identity_transform();
                    let height      = self.get_height_transform(*height as f64);
                    let transform   = CGAffineTransformConcat(identity, height);
                    self.state.set_transform(transform);
                }
                CenterRegion((minx, miny), (maxx, maxy))            => {
                    let current     = self.state.current_transform();
                    let center      = self.get_center_transform(*minx as f64, *miny as f64, *maxx as f64, *maxy as f64);
                    let transform   = CGAffineTransformConcat(current, center);
                    self.state.set_transform(transform);
                }
                MultiplyTransform(transform)                        => {
                    let current                 = self.state.current_transform();
                    let Transform2D(a, b, _c)   = transform;
                    let transform               = CGAffineTransform {
                        a: a.0 as CGFloat,
                        b: b.0 as CGFloat,
                        c: a.1 as CGFloat,
                        d: b.1 as CGFloat,
                        tx: a.2 as CGFloat,
                        ty: a.2 as CGFloat
                    };

                    let transform               = CGAffineTransformConcat(current, transform);
                    self.state.set_transform(transform);
                }
            }
        }
    }
}
