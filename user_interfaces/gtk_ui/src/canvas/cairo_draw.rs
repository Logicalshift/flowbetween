use super::viewport::*;

use flo_canvas as flo;
use flo_canvas::*;

use cairo;
use cairo::*;

///
/// The current source colour that's set
///
#[derive(Copy, Clone, PartialEq)]
enum ColorTarget {
    None,
    Stroke,
    Fill
}

///
/// A saved state in a Cario drawing surface
///
struct SavedState {
    dash_pattern:   Vec<f64>,
    stroke_color:   Color,
    fill_color:     Color
}

impl SavedState {
    ///
    /// Creates a new saved state from a current drawing state
    ///
    pub fn from_current(drawing: &CairoDraw) -> SavedState {
        SavedState {
            dash_pattern:   drawing.dash_pattern.clone(),
            stroke_color:   drawing.stroke_color.clone(),
            fill_color:     drawing.fill_color.clone()
        }
    }

    ///
    /// Restores this saved state into a drawing
    ///
    pub fn restore(self, drawing: &mut CairoDraw) {
        drawing.dash_pattern    = self.dash_pattern;
        drawing.stroke_color    = self.stroke_color;
        drawing.fill_color      = self.fill_color;
        drawing.set_color       = ColorTarget::None;
    }
}

///
/// Represents the current state of a CarioDraw object (used for moving state between layers usually)
///
pub struct CairoState {
    transform:      Matrix,
    line_width:     f64,
    line_join:      cairo::LineJoin,
    line_cap:       cairo::LineCap,
    fill_color:     Color,
    stroke_color:   Color,
    dash_pattern:   Vec<f64>
}

///
/// Performs Flo drawing actions in a Cairo context
///
pub struct CairoDraw {
    /// The context to draw in
    ctxt: Context,

    /// If we consider a 'pixel' as being at a different scale, this is how much bigger a 'real' pixel actually is
    pixel_scale: f64,

    /// The saved states
    saved_states: Vec<SavedState>,

    /// The dash pattern for the next line to draw
    dash_pattern: Vec<f64>,

    /// The current stroke colour
    stroke_color: Color,

    /// The current fill colour
    fill_color: Color,

    /// The colour that's currently set
    set_color: ColorTarget,

    /// The viewport for this canvas
    viewport: CanvasViewport,

    /// The initial translation matrix
    initial_matrix: Matrix
}

impl CairoState {
    ///
    /// Retrieves the transformation matrix attached to this state
    ///
    pub fn get_matrix(&self) -> cairo::Matrix {
        self.transform.clone()
    }
}

impl CairoDraw {
    ///
    /// Creates a new Cairo drawing target
    ///
    pub fn new(ctxt: Context, viewport: CanvasViewport, pixel_scale: f64) -> CairoDraw {
        ctxt.set_matrix(Matrix::from(&viewport));

        CairoDraw {
            ctxt:           ctxt,
            pixel_scale:    pixel_scale,
            saved_states:   vec![],
            dash_pattern:   vec![],
            stroke_color:   Color::Rgba(0.0, 0.0, 0.0, 1.0),
            fill_color:     Color::Rgba(0.0, 0.0, 0.0, 1.0),
            set_color:      ColorTarget::None,
            initial_matrix: Matrix::from(&viewport),
            viewport:       viewport
        }
    }

    ///
    /// Converts a flo line join into a Cairo LineJoin
    ///
    fn get_join(our_join: flo::LineJoin) -> cairo::LineJoin {
        match our_join {
            flo::LineJoin::Miter => cairo::LineJoin::Miter,
            flo::LineJoin::Bevel => cairo::LineJoin::Bevel,
            flo::LineJoin::Round => cairo::LineJoin::Round
        }
    }

    ///
    /// Converts a flo line cap style into a Cairo LineCap
    ///
    fn get_cap(our_cap: flo::LineCap) -> cairo::LineCap {
        match our_cap {
            flo::LineCap::Butt      => cairo::LineCap::Butt,
            flo::LineCap::Round     => cairo::LineCap::Round,
            flo::LineCap::Square    => cairo::LineCap::Square
        }
    }

    ///
    /// Sets the source colour for the specified colour target
    ///
    #[inline]
    fn set_color(&mut self, target: ColorTarget) {
        // Only change the colour if it's not already set
        if self.set_color != target {
            // Get the RGBA components for this target
            let (r, g, b, a) = {
                match target {
                    ColorTarget::None   => (0.0, 0.0, 0.0, 1.0),
                    ColorTarget::Fill   => self.fill_color.to_rgba_components(),
                    ColorTarget::Stroke => self.stroke_color.to_rgba_components()
                }
            };

            // Update the colour with Cairo
            self.ctxt.set_source_rgba(r as f64, g as f64, b as f64, a as f64);

            // Remember that this is the currently set colour
            self.set_color = target;
        }
    }

    ///
    /// Converts a blend mode into an operator
    ///
    fn get_operator(blend: flo::BlendMode) -> cairo::Operator {
        match blend {
            flo::BlendMode::SourceOver      => cairo::Operator::Over,
            flo::BlendMode::SourceIn        => cairo::Operator::In,
            flo::BlendMode::SourceOut       => cairo::Operator::Out,
            flo::BlendMode::DestinationOver => cairo::Operator::DestOver,
            flo::BlendMode::DestinationIn   => cairo::Operator::DestIn,
            flo::BlendMode::DestinationOut  => cairo::Operator::DestOut,
            flo::BlendMode::SourceAtop      => cairo::Operator::Atop,
            flo::BlendMode::DestinationAtop => cairo::Operator::DestAtop,
            flo::BlendMode::Multiply        => cairo::Operator::Multiply,
            flo::BlendMode::Screen          => cairo::Operator::Screen,
            flo::BlendMode::Darken          => cairo::Operator::Darken,
            flo::BlendMode::Lighten         => cairo::Operator::Lighten
        }
    }

    ///
    /// Converts a Flo Transform2D to a Cairo matrix
    ///
    fn get_transform(transform: Transform2D) -> Matrix {
        let Transform2D([a, b, _c]) = transform;

        Matrix::new(a[0] as f64, b[0] as f64, a[1] as f64, b[1] as f64, a[2] as f64, b[2] as f64)
    }

    ///
    /// Sets the line width in pixels
    ///
    fn set_line_width_pixels(&self, pixels: f32) {
        let pixels      = pixels as f64;
        let pixels      = pixels * self.pixel_scale;
        let transform   = self.ctxt.get_matrix();

        // Length of the first column of the transformation matrix is the scale factor (for the width)
        let mut scale = (transform.xx*transform.xx + transform.yx*transform.yx).sqrt();
        if scale == 0.0 { scale = 1.0; }

        // Scale the width down according to this factor (we'll always use the horizontal scale factor)
        let line_width = pixels / scale;
        self.ctxt.set_line_width(line_width);
    }

    ///
    /// Computes the transformation to apply for a particular canvas height
    ///
    fn height_matrix(height: f32) -> Matrix {
        let height      = height as f64;
        let mut ratio_x = 2.0/height;
        let ratio_y     = ratio_x;

        if height < 0.0 {
            ratio_x = -ratio_x;
        }

        let mut result  = Matrix::identity();
        result.scale(ratio_x, ratio_y);

        result
    }

    ///
    /// Computes a matrix to make a particular region centered in the viewport
    ///
    fn center_matrix(current_matrix: &Matrix, viewport: &CanvasViewport, minx: f32, miny: f32, maxx: f32, maxy: f32) -> Matrix {
        let minx            = minx as f64;
        let miny            = miny as f64;
        let maxx            = maxx as f64;
        let maxy            = maxy as f64;

        let pixel_width     = viewport.width as f64;
        let pixel_height    = viewport.height as f64;

        // Get the current scaling of this canvas
        let mut xscale = (current_matrix.xx*current_matrix.xx + current_matrix.yx*current_matrix.yx).sqrt();
        let mut yscale = (current_matrix.xy*current_matrix.xy + current_matrix.yy*current_matrix.yy).sqrt();
        if xscale == 0.0 { xscale = 1.0; }
        if yscale == 0.0 { yscale = 1.0; }

        // Current X, Y coordinates (centered)
        let cur_x = (current_matrix.x0-(pixel_width/2.0))/xscale;
        let cur_y = (current_matrix.y0-(pixel_height/2.0))/yscale;

        // New center coordinates
        let center_x = (minx+maxx)/2.0;
        let center_y = (miny+maxy)/2.0;

        // Compute the offsets and transform the canvas
        let x_offset = cur_x - center_x;
        let y_offset = cur_y - center_y;

        let x_offset = x_offset + (viewport.viewport_x as f64)/xscale;
        let y_offset = y_offset + (viewport.viewport_y as f64)/xscale;

        // Generate the result matrix
        let mut result = Matrix::identity();
        result.translate(x_offset, y_offset);
        result
    }

    ///
    /// Retrieves the transformation matrix for this context
    ///
    pub fn get_matrix(&self) -> cairo::Matrix {
        self.ctxt.get_matrix()
    }

    ///
    /// Copies the drawing state from another CairoDraw object
    ///
    pub fn get_state(&self) -> CairoState {
        let transform       = self.ctxt.get_matrix();
        let line_width      = self.ctxt.get_line_width();
        let line_join       = self.ctxt.get_line_join();
        let line_cap        = self.ctxt.get_line_cap();
        let fill_color      = self.fill_color;
        let stroke_color    = self.stroke_color;
        let dash_pattern    = self.dash_pattern.clone();

        CairoState {
            transform,
            line_width,
            line_join,
            line_cap,
            fill_color,
            stroke_color,
            dash_pattern
        }
    }

    ///
    /// Sets the state of this from a state retrieved via get_state()
    ///
    pub fn set_state(&mut self, state: &CairoState) {
        self.ctxt.set_matrix(state.transform);
        self.ctxt.set_line_width(state.line_width);
        self.ctxt.set_line_join(state.line_join);
        self.ctxt.set_line_cap(state.line_cap);
        self.fill_color     = state.fill_color;
        self.stroke_color   = state.stroke_color;
        self.dash_pattern   = state.dash_pattern.clone();
        self.set_color      = ColorTarget::None;
    }

    ///
    /// Perform a canvas drawing operation in the Cairo context associated with this object
    ///
    pub fn draw(&mut self, drawing: Draw) {
        use self::Draw::*;

        match drawing {
            NewPath                                     => { self.ctxt.new_path(); },
            Move(x, y)                                  => { self.ctxt.move_to(x as f64, y as f64); },
            Line(x, y)                                  => { self.ctxt.line_to(x as f64, y as f64); },
            BezierCurve((x, y), (cx1, cy1), (cx2, cy2)) => { self.ctxt.curve_to(cx1 as f64, cy1 as f64, cx2 as f64, cy2 as f64, x as f64, y as f64); },
            ClosePath                                   => { self.ctxt.close_path(); },
            Fill                                        => { self.set_color(ColorTarget::Fill); self.ctxt.fill_preserve(); },
            Stroke                                      => { self.set_color(ColorTarget::Stroke); self.ctxt.stroke_preserve(); },
            LineWidth(width)                            => { self.ctxt.set_line_width(width as f64); },
            LineWidthPixels(pixels)                     => { self.set_line_width_pixels(pixels); },
            LineJoin(join)                              => { self.ctxt.set_line_join(Self::get_join(join)); },
            LineCap(cap)                                => { self.ctxt.set_line_cap(Self::get_cap(cap)); },
            NewDashPattern                              => { self.dash_pattern = vec![]; self.ctxt.set_dash(&[], 0.0); },
            DashLength(length)                          => { self.dash_pattern.push(length as f64); self.ctxt.set_dash(&self.dash_pattern, self.ctxt.get_dash_offset()); },
            DashOffset(offset)                          => { self.ctxt.set_dash(&self.dash_pattern, offset as f64); },
            FillColor(color)                            => { self.set_color = ColorTarget::None; self.fill_color = color; },
            StrokeColor(color)                          => { self.set_color = ColorTarget::None; self.stroke_color = color; },
            BlendMode(blend)                            => { self.ctxt.set_operator(Self::get_operator(blend)); },
            IdentityTransform                           => { self.ctxt.set_matrix(self.initial_matrix); },
            MultiplyTransform(transform)                => { self.ctxt.transform(Self::get_transform(transform)); },
            Unclip                                      => { self.ctxt.reset_clip(); },
            Clip                                        => { self.ctxt.clip(); },
            Store                                       => { /* Requires external support */ },
            Restore                                     => { /* Requires external support */ },
            FreeStoredBuffer                            => { /* Requires external support */ },
            PushState                                   => { let state = SavedState::from_current(self); self.saved_states.push(state); self.ctxt.save(); },
            PopState                                    => { self.ctxt.restore(); self.saved_states.pop().map(|state| state.restore(self)); },
            Layer(_layer_id)                            => { /* Layers require external support */ },
            LayerBlend(_layer_id, _mode)                => { /* Layers require external support */ },

            CanvasHeight(height)                        => {
                let transform   = self.initial_matrix.clone();
                let height      = Self::height_matrix(height);
                self.ctxt.set_matrix(Matrix::multiply(&height, &transform));
            },
            CenterRegion((minx, miny), (maxx, maxy))    => {
                let transform   = self.ctxt.get_matrix();
                let center      = Self::center_matrix(&transform, &self.viewport, minx, miny, maxx, maxy);
                self.ctxt.set_matrix(Matrix::multiply(&center, &transform));
            },

            ClearCanvas                                 |
            ClearLayer                                  => {
                // Drain any saved states that were created
                let ctxt = &self.ctxt;
                self.saved_states.drain(..).for_each(|_| ctxt.restore());

                // Clear the surface
                self.ctxt.reset_clip();
                self.ctxt.set_operator(Operator::Source);
                self.ctxt.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                self.ctxt.paint();

                // Reset state
                self.fill_color     = Color::Rgba(0.0, 0.0, 0.0, 1.0);
                self.stroke_color   = Color::Rgba(0.0, 0.0, 0.0, 1.0);
                self.set_color      = ColorTarget::None;
                self.dash_pattern   = vec![];

                self.ctxt.set_dash(&[], 0.0);
                self.ctxt.set_line_width(1.0);
                self.ctxt.set_operator(Operator::Over);
                self.ctxt.set_matrix(self.initial_matrix);
            }

            Sprite(sprite_id)                           => { unimplemented!(); },
            ClearSprite                                 => { unimplemented!(); },
            SpriteTransform(transform)                  => { unimplemented!(); },
            DrawSprite(sprite_id)                       => { unimplemented!(); },
        }
    }
}

impl<'a> From<&'a CanvasViewport> for Matrix {
    fn from(viewport: &'a CanvasViewport) -> Matrix {
        let scale           = (viewport.height as f64)/2.0;

        let mut matrix = Matrix::identity();
        matrix.translate(-viewport.viewport_x as f64, -viewport.viewport_y as f64);
        matrix.translate((viewport.width as f64)/2.0, (viewport.height as f64)/2.0);
        matrix.scale(scale, -scale);

        matrix
    }
}

impl From<CanvasViewport> for Matrix {
    #[inline]
    fn from(viewport: CanvasViewport) -> Matrix {
        Matrix::from(&viewport)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn viewport_matrix_center_at_0_0() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, 0.0).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 0.0).1 - 300.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_center_with_offset() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, 0.0).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 0.0).1 - 200.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_center_with_small_window() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     400,
            viewport_height:    300
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, 0.0).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 0.0).1 - 200.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_top_at_0_1() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, -1.0).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, -1.0).1 - 600.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_bottom_at_0_minus_1() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, 1.0).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 1.0).1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_top_with_offset() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);

        assert!((matrix.transform_point(0.0, -1.0).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, -1.0).1 - 500.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_top_bottom_with_custom_height() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix = Matrix::from(viewport);
        let height = CairoDraw::height_matrix(6.0);
        let matrix = Matrix::multiply(&height, &matrix);

        assert!((matrix.transform_point(0.0, -3.0).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, -3.0).1 - 600.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 3.0).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 3.0).1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_top_bottom_translated_with_custom_height() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     400,
            viewport_height:    300
        };
        let matrix = Matrix::from(viewport);
        let height = CairoDraw::height_matrix(6.0);
        let matrix = Matrix::multiply(&height, &matrix);

        assert!((matrix.transform_point(0.0, -3.0).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, -3.0).1 - 500.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 3.0).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(0.0, 3.0).1 - -100.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_recenter_on_rect() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     800,
            viewport_height:    600
        };
        let matrix      = Matrix::from(&viewport);
        let recenter    = CairoDraw::center_matrix(&matrix, &viewport, 2.0, 3.0, 3.0, 4.0);
        let matrix      = Matrix::multiply(&recenter, &matrix);

        assert!((matrix.transform_point(2.5, 3.5).0 - 400.0).abs() < 0.01);
        assert!((matrix.transform_point(2.5, 3.5).1 - 300.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_recenter_on_rect_translated() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     400,
            viewport_height:    300
        };
        let matrix      = Matrix::from(&viewport);
        let recenter    = CairoDraw::center_matrix(&matrix, &viewport, 2.0, 3.0, 3.0, 4.0);
        let matrix      = Matrix::multiply(&recenter, &matrix);

        assert!((matrix.transform_point(2.5, 3.5).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(2.5, 3.5).1 - 200.0).abs() < 0.01);
    }

    #[test]
    fn viewport_matrix_recenter_on_rect_translated_scaled() {
        let viewport = CanvasViewport {
            width:              800,
            height:             600,
            viewport_x:         100,
            viewport_y:         100,
            viewport_width:     400,
            viewport_height:    300
        };
        let mut matrix  = Matrix::from(&viewport);
        matrix.scale(0.5, 0.5);
        let recenter    = CairoDraw::center_matrix(&matrix, &viewport, 2.0, 3.0, 3.0, 4.0);
        let matrix      = Matrix::multiply(&recenter, &matrix);

        assert!((matrix.transform_point(2.5, 3.5).0 - 300.0).abs() < 0.01);
        assert!((matrix.transform_point(2.5, 3.5).1 - 200.0).abs() < 0.01);
    }
}
