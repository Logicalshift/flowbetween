use flo_canvas as flo;
use flo_canvas::*;

use cairo;
use cairo::*;
use cairo::prelude::*;

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
/// Performs Flo drawing actions in a Cairo context
/// 
pub struct CairoDraw {
    /// The context to draw in
    ctxt: Context,

    /// The saved states
    saved_states: Vec<SavedState>,

    /// The dash pattern for the next line to draw
    dash_pattern: Vec<f64>,

    /// The current stroke colour
    stroke_color: Color,

    /// The current fill colour
    fill_color: Color,

    /// The colour that's currently set
    set_color: ColorTarget
}

impl CairoDraw {
    ///
    /// Creates a new Cairo drawing target
    /// 
    pub fn new(ctxt: Context) -> CairoDraw {
        CairoDraw {
            ctxt:           ctxt,
            saved_states:   vec![],
            dash_pattern:   vec![],
            stroke_color:   Color::Rgba(0.0, 0.0, 0.0, 1.0),
            fill_color:     Color::Rgba(0.0, 0.0, 0.0, 1.0),
            set_color:      ColorTarget::None
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
            Fill                                        => { self.set_color(ColorTarget::Fill); self.ctxt.fill(); },
            Stroke                                      => { self.set_color(ColorTarget::Stroke); self.ctxt.stroke(); },
            LineWidth(width)                            => { self.ctxt.set_line_width(width as f64); },
            LineWidthPixels(pixels)                     => {},
            LineJoin(join)                              => { self.ctxt.set_line_join(Self::get_join(join)); },
            LineCap(cap)                                => { self.ctxt.set_line_cap(Self::get_cap(cap)); },
            NewDashPattern                              => { self.dash_pattern = vec![]; self.ctxt.set_dash(&[], 0.0); },
            DashLength(length)                          => { self.dash_pattern.push(length as f64); self.ctxt.set_dash(&self.dash_pattern, self.ctxt.get_dash_offset()); },
            DashOffset(offset)                          => { self.ctxt.set_dash(&self.dash_pattern, offset as f64); },
            FillColor(color)                            => { self.set_color = ColorTarget::None; self.fill_color = color; },
            StrokeColor(color)                          => { self.set_color = ColorTarget::None; self.stroke_color = color; },
            BlendMode(blend)                            => { self.ctxt.set_operator(Self::get_operator(blend)); },
            IdentityTransform                           => {},
            CanvasHeight(height)                        => {},
            CenterRegion((minx, miny), (maxx, maxy))    => {},
            MultiplyTransform(transform)                => {},
            Unclip                                      => { self.ctxt.reset_clip(); },
            Clip                                        => { self.ctxt.clip(); },
            Store                                       => { /* Requires external support */ },
            Restore                                     => { /* Requires external support */ },
            FreeStoredBuffer                            => { /* Requires external support */ },
            PushState                                   => { let state = SavedState::from_current(self); self.saved_states.push(state); self.ctxt.save(); },
            PopState                                    => { self.ctxt.restore(); self.saved_states.pop().map(|state| state.restore(self)); },
            Layer(_layer_id)                            => { /* Layers require external support */ },
            LayerBlend(_layer_id, _mode)                => { /* Layers require external support */ },

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
            }
        }
    }
}
