use super::super::style::*;

use ui::*;
use canvas::*;
use binding::*;

use std::sync::*;

/// Width of an item in a virtualised canvas
const VIRTUAL_WIDTH: f32    = 400.0;

/// Height of the time scale control
const SCALE_HEIGHT: f32     = 24.0;

/// Length of a tick on the timeline (in pixels)
const TICK_LENGTH: f32      = 7.0;

/// Height of a 'main' tick
const TICK_MAIN_HEIGHT: f32 = 10.0;

/// Height of an 'inbetween' tick
const TICK_HEIGHT: f32      = 5.0;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    /// The canvases for the timeline
    canvases:           Arc<ResourceManager<BindingCanvas>>,

    /// A virtual control that draws the timeline scale
    virtual_scale:      VirtualCanvas,

    /// The UI for the timeline
    ui:                 BindRef<Control>
}

impl TimelineController {
    ///
    /// Creates a new timeline controller
    /// 
    pub fn new() -> TimelineController {
        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        // This draws the scale along the top; we use a virtual control as this allows us unlimited width
        let virtual_scale = VirtualCanvas::new(Arc::clone(&canvases), Self::draw_scale);

        // UI
        let virtual_scale_control = virtual_scale.control();
        let ui = BindRef::new(&computed(move || Control::scrolling_container()
            .with(Bounds::fill_all())
            .with(Scroll::MinimumContentSize(6000.0, 256.0))
            .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Always))
            .with(Scroll::VerticalScrollBar(ScrollBarVisibility::OnlyIfNeeded))
            .with(Appearance::Background(TIMELINE_BACKGROUND))
            .with(vec![
                Control::container()
                    .with(Bounds {
                        x1: Position::At(0.0),
                        x2: Position::End,
                        y1: Position::At(0.0),
                        y2: Position::At(SCALE_HEIGHT)
                    })
                    .with(vec![
                        virtual_scale_control.get()
                    ])
            ])
            .with((ActionTrigger::VirtualScroll(VIRTUAL_WIDTH, 256.0), "Scroll"))));

        // Piece it together
        TimelineController {
            ui:             ui,
            virtual_scale:  virtual_scale,
            canvases:       canvases
        }
    }

    ///
    /// Draws the timeline scale
    /// 
    fn draw_scale(gc: &mut GraphicsPrimitives, (x, _y): (f32, f32)) {
        // Set up the canvas
        gc.canvas_height(SCALE_HEIGHT);
        gc.center_region(x, 0.0, x+VIRTUAL_WIDTH, SCALE_HEIGHT);
        gc.line_width(1.0);

        // Draw the ticks
        let start_tick  = (x / TICK_LENGTH).floor() as i32;
        let end_tick    = ((x+VIRTUAL_WIDTH)/TICK_LENGTH).ceil() as i32;

        gc.stroke_color(TIMESCALE_TICK);

        for tick in start_tick..(end_tick+1) {
            let tick_x = (tick as f32) * TICK_LENGTH;
            let tick_x = tick_x + (TICK_LENGTH/2.0);

            gc.new_path();

            if (tick%5) == 0 {
                gc.stroke_color(TIMESCALE_MAINTICK);

                gc.move_to(tick_x, 0.0);
                if (tick%10) == 0 {
                    gc.line_to(tick_x, TICK_MAIN_HEIGHT);
                } else {
                    gc.line_to(tick_x, TICK_HEIGHT);
                }
                gc.stroke();

                gc.stroke_color(TIMESCALE_TICK);
            } else {
                gc.move_to(tick_x, 0.0);
                gc.line_to(tick_x, TICK_HEIGHT);
                gc.stroke();
            }
        }

        // Draw the border line
        gc.stroke_color(TIMESCALE_BORDER);
        gc.new_path();
        gc.move_to(x, 0.5);
        gc.line_to(x+VIRTUAL_WIDTH, 0.5);
        gc.stroke();
    }
}

impl Controller for TimelineController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(Arc::clone(&self.canvases))
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            ("Scroll", &VirtualScroll((x, _y), (width, _height))) => {
                // The virtual scale is always drawn at the top, so we hard-code the top and height values
                // Expanding the grid width by 2 allows for a 'buffer' on either side to prevent pop-in
                let virtual_x = if x > 0 { x-1 } else { x };
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, SCALE_HEIGHT), (virtual_x, 0), (width+2, 1));
            },

            _ => ()
        }
    }
}
