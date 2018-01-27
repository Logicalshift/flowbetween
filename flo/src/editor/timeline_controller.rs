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
const TICK_LENGTH: f32      = 8.0;

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
            .with(ControlAttribute::Padding((16, 16), (16, 16)))
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

        // TODO: draw the ticks

        // Draw the border line
        gc.stroke_color(TIMESCALE_BORDER);
        gc.new_path();
        gc.move_to(x, SCALE_HEIGHT-0.5);
        gc.line_to(x+VIRTUAL_WIDTH, SCALE_HEIGHT-0.5);
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
            ("Scroll", &VirtualScroll((x, y), (width, height))) => {
                // The virtual scale is always drawn at the top, so we hard-code the top and height values
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, SCALE_HEIGHT), (x, 0), (width, 1));

                println!("{:?} {:?}", (x, y), (width, height));
            },

            _ => ()
        }
    }
}
