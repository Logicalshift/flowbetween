use crate::scenery::ui::colors::*;

use flo_draw::canvas::*;
use flo_curves::bezier::*;
use flo_curves::arc::*;

use std::f64;

///
/// Possible states of a tool 'plinth' (the background we draw under a tool)
///
pub enum ToolPlinthState {
    Unselected,
    Highlighted,
    Selected,
    StartDrag(f64),
}

///
/// Extra primitives for rendering tools
///
pub trait ToolGraphicsPrimitives {
    ///
    /// Adds a rounded rectangle to the current path
    ///
    fn rounded_rect(&mut self, pos: (f32, f32), size: (f32, f32), radius: f32);

    ///
    /// Draws a tool dock
    ///
    fn tool_dock(&mut self, pos: (f32, f32), size: (f32, f32));

    ///
    /// Draws a tool 'plinth' (the background rendered until the icon for a tool)
    ///
    fn tool_plinth(&mut self, pos: (f32, f32), size: (f32, f32), state: ToolPlinthState);
}

impl<T> ToolGraphicsPrimitives for T
where
    T : GraphicsContext,
{
    fn rounded_rect(&mut self, pos: (f32, f32), size: (f32, f32), radius: f32) {
        const R0: f64 = 0.0;
        const R1: f64 = f64::consts::PI * 0.5;
        const R2: f64 = f64::consts::PI * 1.0;
        const R3: f64 = f64::consts::PI * 1.5;
        const R4: f64 = f64::consts::PI * 2.0;

        let (x, y) = pos;
        let (w, h) = size;

        let radius = (w/2.0).min(h/2.0).min(radius);

        self.move_to(x, y + radius);
        self.line_to(x, y + h - radius);
        self.bezier_curve(&Circle::new(Coord2((x + radius) as _, (y + h - radius) as _), radius as _).arc(R3, R4).to_bezier_curve::<Curve<_>>());
        self.line_to(x + w - radius, y + h);
        self.bezier_curve(&Circle::new(Coord2((x + w - radius) as _, (y + h - radius) as _), radius as _).arc(R0, R1).to_bezier_curve::<Curve<_>>());
        self.line_to(x + w, y + radius);
        self.bezier_curve(&Circle::new(Coord2((x + w - radius) as _, (y + radius) as _), radius as _).arc(R1, R2).to_bezier_curve::<Curve<_>>());
        self.line_to(x + radius, y);
        self.bezier_curve(&Circle::new(Coord2((x + radius) as _, (y + radius) as _), radius as _).arc(R2, R3).to_bezier_curve::<Curve<_>>());

        self.close_path();
    }

    fn tool_dock(&mut self, pos: (f32, f32), size: (f32, f32)) {
        self.push_state();

        self.new_path();
        self.rounded_rect(pos, size, 12.0);
        self.fill_color(color_tool_dock_background());
        self.fill();

        self.new_path();
        self.rounded_rect((pos.0+2.0, pos.1+2.0), (size.0-4.0, size.1-4.0), 10.0);
        self.line_width(1.0);
        self.stroke_color(color_tool_dock_outline());
        self.stroke();

        self.pop_state();
    }

    fn tool_plinth(&mut self, pos: (f32, f32), size: (f32, f32), state: ToolPlinthState) {
        self.push_state();

        match state {
            ToolPlinthState::Unselected => { }

            ToolPlinthState::Highlighted => {
                self.new_path();
                self.rounded_rect(pos, size, 8.0);
                self.fill_color(color_tool_dock_highlight());
                self.fill();
            }

            ToolPlinthState::Selected => {
                self.new_path();
                self.rounded_rect(pos, size, 8.0);
                self.fill_color(color_tool_dock_selected());
                self.fill();

                self.new_path();
                self.rounded_rect((pos.0+1.0, pos.1+1.0), (size.0-2.0, size.1-2.0), 7.0);
                self.stroke_color(color_tool_dock_outline());
                self.line_width(1.0);
                self.stroke();
            }

            ToolPlinthState::StartDrag(ratio) => {
                let width       = size.0;
                let rounding    = ((width / 2.0) - 8.0) * (ratio as f32) + 8.0;
                let fill_color  = color_tool_dock_highlight().with_alpha(ratio as _);

                self.new_path();
                self.rounded_rect(pos, size, rounding);

                self.fill_color(fill_color);
                self.fill();

                self.stroke_color(color_tool_dock_outline());
                self.line_width(1.0);
                self.stroke();
            }
        }

        self.pop_state();
    }
}
