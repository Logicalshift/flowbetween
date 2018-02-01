use super::super::style::*;
use super::super::viewmodel::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;

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
pub struct TimelineController<Anim: Animation> {
    /// The view model for this controller
    _view_model:        AnimationViewModel<Anim>,

    /// The canvases for the timeline
    canvases:           Arc<ResourceManager<BindingCanvas>>,

    /// A virtual control that draws the timeline scale
    virtual_scale:      VirtualCanvas,

    /// A virtual control that draws the keyframes
    virtual_keyframes:  VirtualCanvas,

    /// The UI for the timeline
    ui:                 BindRef<Control>
}

impl<Anim: 'static+Animation> TimelineController<Anim> {
    ///
    /// Creates a new timeline controller
    /// 
    pub fn new(anim_view_model: &AnimationViewModel<Anim>) -> TimelineController<Anim> {
        let anim_view_model = anim_view_model.clone();

        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        // This draws the scale along the top; we use a virtual control as this allows us unlimited width
        let virtual_scale = VirtualCanvas::new(Arc::clone(&canvases), Self::draw_scale);

        // This draws the keyframes
        let virtual_keyframes = VirtualCanvas::new(Arc::clone(&canvases), Self::draw_scale);

        // UI
        let duration        = BindRef::new(&anim_view_model.timeline().duration);
        let frame_duration  = BindRef::new(&anim_view_model.timeline().frame_duration);

        let virtual_scale_control       = virtual_scale.control();
        let virtual_keyframes_control   = virtual_keyframes.control();
        let ui = BindRef::new(&computed(move || {
            // Work out the number of frames in this animation
            let duration            = duration.get();
            let frame_duration      = frame_duration.get();

            let duration_ns         = duration.as_secs()*1_000_000_000 + (duration.subsec_nanos() as u64);
            let frame_duration_ns   = frame_duration.as_secs()*1_000_000_000 + (frame_duration.subsec_nanos() as u64);

            let width               = TICK_LENGTH * ((duration_ns / frame_duration_ns) as f32);

            // Build the final control
            Control::scrolling_container()
                .with(Bounds::fill_all())
                .with(Scroll::MinimumContentSize(width, 256.0))
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
                        ]),
                    Control::container()
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::End,
                            y1: Position::At(SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(vec![
                            virtual_keyframes_control.get()
                        ])
                ])
                .with((ActionTrigger::VirtualScroll(VIRTUAL_WIDTH, 256.0), "Scroll"))
        }));

        // Piece it together
        TimelineController {
            _view_model:        anim_view_model,
            ui:                 ui,
            virtual_scale:      virtual_scale,
            virtual_keyframes:  virtual_keyframes,
            canvases:           canvases
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

impl<Anim: Animation> Controller for TimelineController<Anim> {
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
                // Expanding the grid width by 2 allows for a 'buffer' on either side to prevent pop-in
                let virtual_x = if x > 0 { x-1 } else { x };
                let virtual_y = if y > 0 { y-1 } else { y };
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, SCALE_HEIGHT), (virtual_x, 0), (width+2, 1));
                self.virtual_keyframes.virtual_scroll((VIRTUAL_WIDTH, 256.0), (virtual_x, virtual_y), (width+2, height+2));
            },

            _ => ()
        }
    }
}
