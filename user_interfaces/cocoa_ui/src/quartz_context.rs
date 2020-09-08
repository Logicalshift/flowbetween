use super::canvas_state::*;
use super::canvas_transform::*;
use super::core_graphics_ffi::*;

use flo_canvas::*;

///
/// Processes canvas draw commands onto a core graphics context
///
/// This assumes that all the commands are intended for a specific layer in the canvas: ie, layer switch commands
/// are ignored.
///
pub struct QuartzContext {
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

impl QuartzContext {
    ///
    /// Creates a new canvas layer that will render to the specified context
    ///
    pub unsafe fn new(context: CFRef<CGContextRef>, viewport_origin: (f64, f64), viewport_size: (f64, f64), canvas_size: (f64, f64)) -> QuartzContext {
        // Colours are in the SRGB colourspace
        let srgb        = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        let mut state   = CanvasState::new(CFRef::from(srgb));

        state.activate_context(context.clone());

        let mut new_layer = QuartzContext {
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
    /// Resets the state back to the default
    ///
    fn reset_state(&mut self) {
        unsafe {
            // Deactivate the original state (which will also clear any stack associated with it)
            self.state.deactivate_context();

            // Create a new state
            let srgb    = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
            let state   = CanvasState::new(CFRef::from(srgb));

            // Activate the new state
            self.state = state;
            self.state.activate_context(self.context.clone());

            // Reset the transform to the identity transform
            self.state.set_transform(self.get_identity_transform());
        }
    }

    ///
    /// Updates the state of this context
    ///
    pub fn set_state(&mut self, new_state: CanvasState) {
        self.state.deactivate_context();
        self.state = new_state;
        self.state.activate_context(self.context.clone());
    }

    ///
    /// Destroys this context and returns the current state
    ///
    pub fn to_state(mut self) -> CanvasState {
        self.state.deactivate_context();
        self.state
    }

    ///
    /// Retrieves a reference to the state in this context
    ///
    pub fn get_state<'a>(&'a mut self) -> &'a mut CanvasState {
        &mut self.state
    }

    ///
    /// Computes the identity transform for this canvas
    ///
    fn get_identity_transform(&self) -> CGAffineTransform {
        canvas_identity_transform(self.viewport_origin, self.canvas_size)
    }

    ///
    /// Computes a matrix to be appended to the identity transform that will set the height of the canvas
    ///
    fn get_height_transform(&self, height: f64) -> CGAffineTransform {
        canvas_height_transform(height)
    }

    ///
    /// Retrieves the transformation needed to move the center of the canvas to the specified point
    ///
    fn get_center_transform(&self, minx: f64, miny: f64, maxx: f64, maxy: f64) -> CGAffineTransform {
        canvas_center_transform(self.viewport_origin, self.canvas_size, self.state.current_transform(), minx, miny, maxx, maxy)
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&mut self, draw: &Draw) {
        use self::Draw::*;

        unsafe {
            match draw {
                NewPath                                             => { self.state.begin_path(); }
                Move(x, y)                                          => { self.state.path_move(*x as CGFloat, *y as CGFloat); }
                Line(x, y)                                          => { self.state.path_line(*x as CGFloat, *y as CGFloat); }
                BezierCurve((ex, ey), (c1x, c1y), (c2x, c2y))       => { self.state.path_bezier_curve((*c1x as CGFloat, *c1y as CGFloat), (*c2x as CGFloat, *c2y as CGFloat), (*ex as CGFloat, *ey as CGFloat)); }
                ClosePath                                           => { self.state.path_close(); }
                Fill                                                => { self.state.load_path(); CGContextFillPath(*self.context); }
                Stroke                                              => { self.state.load_path(); CGContextStrokePath(*self.context); }
                LineWidth(width)                                    => { self.state.set_line_width(*width as CGFloat); }
                LineWidthPixels(width_pixels)                       => {
                    let width_pixels    = *width_pixels as CGFloat;
                    let transform       = self.state.current_transform();
                    let mut scale_y     = (transform.c*transform.c + transform.d*transform.d).sqrt();
                    if scale_y == 0.0 { scale_y = 1.0 }
                    let scale_width     = width_pixels / scale_y;

                    self.state.set_line_width(scale_width);
                }
                LineJoin(join)                                      => { self.state.set_line_join(join); }
                LineCap(cap)                                        => { self.state.set_line_cap(cap); }
                NewDashPattern                                      => { /* TODO */ }
                DashLength(len)                                     => { /* TODO */ }
                DashOffset(offset)                                  => { /* TODO */ }
                FillColor(col)                                      => { self.state.set_fill_color(col); }
                StrokeColor(col)                                    => { self.state.set_stroke_color(col); }
                BlendMode(blend)                                    => { self.state.set_blend_mode(blend); }
                Unclip                                              => { self.state.unclip(); }
                Clip                                                => { self.state.clip(); }
                Store                                               => { /* See view_canvas */ }
                Restore                                             => { /* See view_canvas */ }
                FreeStoredBuffer                                    => { /* See view_canvas */ }
                PushState                                           => { self.state.push_state(); }
                PopState                                            => { self.state.pop_state(); }

                ClearLayer                                          => {
                    let viewport_size = self.viewport_size;
                    self.state.with_native_transform(|context| {
                        CGContextClearRect(context, CGRect {
                            origin: CGPoint { x: 0.0, y: 0.0 },
                            size:   CGSize  { width: viewport_size.0 as CGFloat, height: viewport_size.1 as CGFloat }
                        })
                    });
                }

                IdentityTransform                                   => {
                    self.state.set_transform(self.get_identity_transform());
                }
                CanvasHeight(height)                                => {
                    let identity    = self.get_identity_transform();
                    let height      = self.get_height_transform(*height as f64);
                    let transform   = CGAffineTransformConcat(height, identity);
                    self.state.set_transform(transform);
                }
                CenterRegion((minx, miny), (maxx, maxy))            => {
                    let current     = self.state.current_transform();
                    let center      = self.get_center_transform(*minx as f64, *miny as f64, *maxx as f64, *maxy as f64);
                    let transform   = CGAffineTransformConcat(center, current);
                    self.state.set_transform(transform);
                }
                MultiplyTransform(transform)                        => {
                    let current                 = self.state.current_transform();
                    let Transform2D([a, b, _c]) = transform;
                    let transform               = CGAffineTransform {
                        a: a[0] as CGFloat,
                        b: b[0] as CGFloat,
                        c: a[1] as CGFloat,
                        d: b[1] as CGFloat,
                        tx: a[2] as CGFloat,
                        ty: b[2] as CGFloat
                    };

                    let transform               = CGAffineTransformConcat(transform, current);
                    self.state.set_transform(transform);
                }

                ClearCanvas                                         => {
                    self.reset_state();

                    /* Layers need to be implemented elsewhere */
                }
                Layer(_layer_id)                                    => { /* Layers need to be implemented elsewhere */ }
                LayerBlend(_layer_id, _blend)                       => { /* Layers need to be implemented elsewhere */ }
                Sprite(_sprite_id)                                  => { unimplemented!() }
                SpriteTransform(_transform)                         => { unimplemented!() }
                ClearSprite                                         => { unimplemented!() }
                DrawSprite(_sprite_id)                              => { unimplemented!() }
            }
        }
    }
}
