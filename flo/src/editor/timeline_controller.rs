use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::time::Duration;

/// Action when the user drags the timeline 'time' indicator
const DRAG_TIMELINE_POSITION: &str = "DragTime";

/// Action when the user clicks/drags on the scale away from the 'time' indicator
const CLICK_AND_DRAG_TIMELINE_POSITION: &str = "ClickTime";

/// Action when the virtual scroll position changes
const SCROLL_TIMELINE: &str     = "Scroll";

/// Width of an item in a virtualised canvas
const VIRTUAL_WIDTH: f32        = 400.0;

/// Height of an item in a virtualised canvas
const VIRTUAL_HEIGHT: f32       = 256.0;

/// Height of the time scale control
const SCALE_HEIGHT: f32         = 24.0;

/// Length of a tick on the timeline (in pixels)
const TICK_LENGTH: f32          = 7.0;

/// Height of a 'main' tick
const TICK_MAIN_HEIGHT: f32     = 10.0;

/// Height of an 'inbetween' tick
const TICK_HEIGHT: f32          = 5.0;

/// Height of a layer in pixels
const LAYER_HEIGHT: f32         = 24.0;

/// Width of the layer name panel
const LAYER_PANEL_WIDTH: f32    = 256.0;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController<Anim: Animation> {
    /// The view model for this controller
    anim_model:         FloModel<Anim>,

    /// The UI view model
    view_model:         Arc<DynamicViewModel>,

    /// The canvases for the timeline
    canvases:           Arc<ResourceManager<BindingCanvas>>,

    /// The current_time at the most recent drag start position
    drag_start_time:    Binding<Duration>,

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
    pub fn new(anim_model: &FloModel<Anim>) -> TimelineController<Anim> {
        let anim_model = anim_model.clone();

        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        // This draws the scale along the top; we use a virtual control as this allows us unlimited width
        let virtual_scale = VirtualCanvas::new(Arc::clone(&canvases), Self::draw_scale);

        // This draws the keyframes
        let create_keyframe_canvas  = Self::create_draw_keyframes_fn(anim_model.timeline());
        let virtual_keyframes       = VirtualCanvas::new(Arc::clone(&canvases), move |x, y| (create_keyframe_canvas)(x, y));

        // Viewmodel specifies a few dynamic things
        let view_model = DynamicViewModel::new();

        // Indicator xpos is computed from the current frame
        let current_time    = anim_model.timeline().current_time.clone();
        let frame_duration  = anim_model.timeline().frame_duration.clone();
        view_model.set_computed("IndicatorXPos", move || {
            let current_time        = current_time.get();
            let frame_duration      = frame_duration.get();

            let current_time_ns     = current_time.as_secs() * 1_000_000_000 + (current_time.subsec_nanos() as u64);
            let frame_duration_ns   = frame_duration.as_secs() * 1_000_000_000 + (frame_duration.subsec_nanos() as u64);

            let frame               = current_time_ns / frame_duration_ns;

            let tick_length         = TICK_LENGTH as f64;
            let tick_x              = (frame as f64) * tick_length;
            let tick_x              = tick_x + (tick_length/2.0);
            let tick_x              = tick_x + (LAYER_PANEL_WIDTH as f64);

            PropertyValue::Float(tick_x)
        });

        // UI
        let duration        = BindRef::new(&anim_model.timeline().duration);
        let frame_duration  = BindRef::new(&anim_model.timeline().frame_duration);
        let layers          = BindRef::new(&anim_model.timeline().layers);

        let virtual_scale_control       = virtual_scale.control();
        let virtual_keyframes_control   = virtual_keyframes.control();

        let ui = Self::ui(layers, duration, frame_duration, virtual_scale_control, virtual_keyframes_control, Arc::clone(&canvases));

        // Piece it together
        TimelineController {
            anim_model:         anim_model,
            ui:                 ui,
            virtual_scale:      virtual_scale,
            virtual_keyframes:  virtual_keyframes,
            drag_start_time:    bind(Duration::from_millis(0)),
            canvases:           canvases,
            view_model:         Arc::new(view_model)
        }
    }

    ///
    /// Creates the user interface for the timeline
    /// 
    fn ui(layers: BindRef<Vec<LayerModel>>, duration: BindRef<Duration>, frame_duration: BindRef<Duration>, virtual_scale_control: BindRef<Control>, virtual_keyframes_control: BindRef<Control>, canvases: Arc<ResourceManager<BindingCanvas>>) -> BindRef<Control> {
        let timescale_indicator         = BindingCanvas::with_drawing(Self::draw_frame_indicator);
        let timescale_indicator         = canvases.register(timescale_indicator);

        let timescale_indicator_line    = BindingCanvas::with_drawing(Self::draw_frame_indicator_line);
        let timescale_indicator_line    = canvases.register(timescale_indicator_line);

        BindRef::new(&computed(move || {
            let timescale_indicator         = timescale_indicator.clone();
            let timescale_indicator_line    = timescale_indicator_line.clone();
            
            // Work out the number of frames in this animation
            let duration            = duration.get();
            let frame_duration      = frame_duration.get();
            let layers              = layers.get();

            let duration_ns         = duration.as_secs()*1_000_000_000 + (duration.subsec_nanos() as u64);
            let frame_duration_ns   = frame_duration.as_secs()*1_000_000_000 + (frame_duration.subsec_nanos() as u64);

            let width               = TICK_LENGTH * ((duration_ns / frame_duration_ns) as f32);
            let height              = (layers.len() as f32) * LAYER_HEIGHT;

            // Build the final control
            Control::scrolling_container()
                .with(Bounds::fill_all())
                .with(Scroll::MinimumContentSize(width, height + SCALE_HEIGHT))
                .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Always))
                .with(Scroll::VerticalScrollBar(ScrollBarVisibility::OnlyIfNeeded))
                .with(Appearance::Background(TIMELINE_BACKGROUND))
                .with(vec![
                    Control::container()        // Layer editor
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::At(LAYER_PANEL_WIDTH),
                            y1: Position::At(0.0),
                            y2: Position::End
                        })
                        .with(vec![
                            Control::empty()
                                .with(Bounds::stretch_horiz(1.0)),
                            Control::empty()
                                .with(Bounds::next_horiz(1.0))
                                .with(Appearance::Background(TIMESCALE_BORDER))
                        ])
                        .with(Appearance::Background(TIMELINE_BACKGROUND))
                        .with(Scroll::Fix(FixedAxis::Horizontal))
                        .with(ControlAttribute::ZIndex(5)),
                    Control::container()        // Scale
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::End,
                            y1: Position::At(0.0),
                            y2: Position::At(SCALE_HEIGHT)
                        })
                        .with((ActionTrigger::Drag, CLICK_AND_DRAG_TIMELINE_POSITION))
                        .with(ControlAttribute::ZIndex(3))
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with(vec![
                            virtual_scale_control.get()
                        ]),
                    Control::container()        // Timeline
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::End,
                            y1: Position::At(SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(vec![
                            virtual_keyframes_control.get()
                        ])
                        .with(ControlAttribute::ZIndex(2)),
                    Control::canvas()           // Selected frame indicator (upper part, arrow indicator)
                        .with(timescale_indicator)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 16.0),
                            y1: Position::Start,
                            y2: Position::At(SCALE_HEIGHT)
                        })
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with((ActionTrigger::Drag, DRAG_TIMELINE_POSITION))
                        .with(ControlAttribute::ZIndex(4)),
                    Control::canvas()           // Selected frame indicator (lower part, under the timeline)
                        .with(timescale_indicator_line)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 16.0),
                            y1: Position::At(SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(ControlAttribute::ZIndex(1))
                ])
                .with((ActionTrigger::VirtualScroll(VIRTUAL_WIDTH, VIRTUAL_HEIGHT), SCROLL_TIMELINE))
        }))
    }

    ///
    /// Creates the function for drawing the keyframes
    /// 
    fn create_draw_keyframes_fn(timeline: &TimelineModel<Anim>) -> impl Fn(f32, f32) -> Box<dyn Fn(&mut dyn GraphicsPrimitives) -> ()+Send+Sync>+Send+Sync {
        let timeline    = timeline.clone();

        move |x, y| {
            // Get the layers that we'll draw
            let first_layer = (y/VIRTUAL_HEIGHT).floor() as u32;
            let last_layer  = ((y+VIRTUAL_HEIGHT)/VIRTUAL_HEIGHT).ceil() as u32 + 1;

            // ... and the keyframes in this time region
            let tick_x      = x - LAYER_PANEL_WIDTH;
            let start_tick  = (tick_x/TICK_LENGTH).floor();
            let end_tick    = ((tick_x+VIRTUAL_WIDTH)/TICK_LENGTH).ceil() + 1.0;
            let start_tick  = start_tick.max(0.0) as u32;
            let end_tick    = end_tick.max(0.0) as u32;
            let keyframes   = timeline.get_keyframe_binding(start_tick..end_tick);
            let layers      = BindRef::new(&timeline.layers);

            // Generate the drawing function for this part of the canvas
            Box::new(move |gc| {
                let layers      = layers.get();
                let keyframes   = keyframes.get();

                let last_layer  = last_layer.min(layers.len() as u32);
                let end_tick    = end_tick;

                // Center the drawing region
                gc.canvas_height(-VIRTUAL_HEIGHT);
                gc.center_region(x, y, x+VIRTUAL_WIDTH, y+VIRTUAL_HEIGHT);

                gc.line_width(0.5);

                // Draw the cell dividers
                let end_x = (end_tick as f32) * TICK_LENGTH;
                let end_x = end_x + LAYER_PANEL_WIDTH;
                let end_y = (last_layer as f32) * LAYER_HEIGHT;

                gc.stroke_color(TIMESCALE_CELL);

                gc.new_path();
                for cell_index in start_tick..end_tick {
                    let cell_x = (cell_index as f32) * TICK_LENGTH;
                    let cell_x = cell_x + TICK_LENGTH/2.0;
                    let cell_x = cell_x + LAYER_PANEL_WIDTH;

                    gc.move_to(cell_x, y);
                    gc.line_to(cell_x, end_y);
                }
                gc.stroke();

                // Draw the layer dividers
                gc.stroke_color(TIMESCALE_LAYERS);

                gc.new_path();
                for layer_index in first_layer..last_layer {
                    let layer_y = ((layer_index as f32) * LAYER_HEIGHT) - 0.5;

                    gc.move_to(x, layer_y + LAYER_HEIGHT);
                    gc.line_to(end_x, layer_y + LAYER_HEIGHT);
                }
                gc.stroke();

                // Draw the keyframes that are in this region
                gc.fill_color(TIMESCALE_KEYFRAME);
                for keyframe in keyframes.iter() {
                    // Fetch where this frame occurs
                    let frame       = keyframe.frame;
                    let layer_id    = keyframe.layer_id;
                    //let layer_index = layers.iter().filter(|layer| layer.id.get() == layer_id).nth(0);
                    let layer_index = Some(layer_id as u32); // TODO: need the index, not the ID but we'll use the ID for now

                    // Draw it if it's in this view
                    if let Some(layer_index) = layer_index {
                        if layer_index >= first_layer && layer_index < last_layer {
                            // Top-left corner of this frame
                            let xpos = (frame as f32) * TICK_LENGTH;
                            let xpos = xpos + LAYER_PANEL_WIDTH;
                            let ypos = (layer_index as f32) * LAYER_HEIGHT;

                            // Draw the frame marker
                            gc.new_path();
                            gc.circle(xpos + TICK_LENGTH/2.0, ypos + LAYER_HEIGHT/2.0, TICK_LENGTH/2.0 - 0.5);
                            gc.fill();
                        }
                    }
                }
            })
        }
    }

    ///
    /// Draws the timeline scale
    /// 
    fn draw_scale(x: f32, _y: f32) -> Box<dyn Fn(&mut dyn GraphicsPrimitives) -> ()+Send+Sync> {
        Box::new(move |gc| {
            // Set up the canvas
            gc.canvas_height(SCALE_HEIGHT);
            gc.center_region(x, 0.0, x+VIRTUAL_WIDTH, SCALE_HEIGHT);
            gc.line_width(1.0);

            // Fill the background
            gc.fill_color(TIMESCALE_BACKGROUND);
            gc.new_path();
            gc.rect(x, 0.0, x+VIRTUAL_WIDTH, SCALE_HEIGHT);
            gc.fill();

            // Draw the ticks
            let tick_x      = x - LAYER_PANEL_WIDTH;
            let start_tick  = (tick_x / TICK_LENGTH).floor();
            let start_tick  = start_tick.max(0.0);
            let end_tick    = ((tick_x+VIRTUAL_WIDTH)/TICK_LENGTH).ceil();
            let end_tick    = end_tick.max(0.0);

            let start_tick  = start_tick as i32;
            let end_tick    = end_tick as i32;

            gc.stroke_color(TIMESCALE_TICK);

            for tick in start_tick..(end_tick+1) {
                let tick_x = (tick as f32) * TICK_LENGTH;
                let tick_x = tick_x + (TICK_LENGTH/2.0);
                let tick_x = tick_x + LAYER_PANEL_WIDTH;

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
        })
    }

    ///
    /// Draws the frame indicator
    /// 
    fn draw_frame_indicator(gc: &mut dyn GraphicsPrimitives) -> () {
        gc.canvas_height(2.05);

        gc.fill_color(TIMESCALE_INDICATOR);

        gc.new_path();
        gc.circle(0.0, 0.2, 0.6);
        gc.fill();

        gc.new_path();
        gc.move_to(-0.6, 0.2);
        gc.line_to(0.0, -1.0);
        gc.line_to(0.6, 0.2);
        gc.close_path();
        gc.fill();
    }

    ///
    /// Draws the frame indicator line
    /// 
    fn draw_frame_indicator_line(gc: &mut dyn GraphicsPrimitives) -> () {
        gc.stroke_color(TIMESCALE_INDICATOR2);
        gc.canvas_height(2.0);
        gc.line_width_pixels(1.0);

        gc.new_path();
        gc.move_to(0.0, -1.0);
        gc.line_to(0.0, 1.0);
        gc.stroke();
    }

    ///
    /// Converts a duration to ns
    /// 
    fn duration_to_ns(time: Duration) -> i64 {
        (time.as_secs() * 1_000_000_000) as i64 + (time.subsec_nanos() as i64)
    }

    ///
    /// Converts a nanosecond time to a duration
    /// 
    /// Durations can't represent negative values, so negative times will be represented
    /// as 0
    /// 
    fn ns_to_duration(ns: i64) -> Duration {
        if ns < 0 {
            Duration::new(0, 0)
        } else {
            Duration::new((ns / 1_000_000_000) as u64, (ns % 1_000_000_000) as u32)
        }
    }

    ///
    /// Converts an x position to a nanosecond value
    /// 
    /// (Nanoseconds rather than a duration so this works for negative times)
    /// 
    fn xpos_to_ns(&self, xpos: f32) -> i64 {
        // Get the frame duration and start time in nanoseconds
        let timeline            = self.anim_model.timeline();
        let frame_duration      = timeline.frame_duration.get();

        let frame_duration_ns   = Self::duration_to_ns(frame_duration);

        // Work out the number of frames from the start we are
        let frames              = (xpos / TICK_LENGTH).round() as i64;
        let time_ns             = frames * (frame_duration_ns as i64);

        time_ns
    }
}

impl<Anim: Animation+'static> Controller for TimelineController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(Arc::clone(&self.canvases))
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use self::ActionParameter::*;

        match (action_id, action_parameter) {
            (SCROLL_TIMELINE, &VirtualScroll((x, y), (width, height))) => {
                // The virtual scale is always drawn at the top, so we hard-code the top and height values
                // Expanding the grid width by 2 allows for a 'buffer' on either side to prevent pop-in
                let virtual_x = if x > 0 { x-1 } else { x };
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, SCALE_HEIGHT), (virtual_x, 0), (width+2, 1));
                self.virtual_keyframes.virtual_scroll((VIRTUAL_WIDTH, VIRTUAL_HEIGHT), (virtual_x, y), (width+2, height));
            },

            (CLICK_AND_DRAG_TIMELINE_POSITION, &Drag(DragAction::Start, (start_x, _start_y), _)) => {
                // Clicking on the scale moves the time to where the user clicked initially
                let time_ns = self.xpos_to_ns(start_x);
                let time    = Self::ns_to_duration(time_ns);

                self.anim_model.timeline().current_time.clone().set(time);
                self.drag_start_time.clone().set(time);
            },

            (DRAG_TIMELINE_POSITION, &Drag(DragAction::Start, _, _)) => {
                // Remember the start time when a drag begins
                self.drag_start_time.clone().set(self.anim_model.timeline().current_time.get());
            },

            (DRAG_TIMELINE_POSITION, &Drag(_drag_type, (start_x, _start_y), (x, _y)))
            | (CLICK_AND_DRAG_TIMELINE_POSITION, &Drag(_drag_type, (start_x, _start_y), (x, _y))) => {
                // Get the frame duration and start time in nanoseconds
                let timeline            = self.anim_model.timeline();
                let start_time          = self.drag_start_time.get();

                let start_time_ns       = Self::duration_to_ns(start_time);

                // Work out the number of frames from the start we are
                let diff_x              = x - start_x;
                let diff_time_ns        = self.xpos_to_ns(diff_x);

                // New time from nanoseconds
                let new_time            = Self::ns_to_duration(start_time_ns + diff_time_ns);

                // Update the viewmodel time
                timeline.current_time.clone().set(new_time);
            },

            _ => ()
        }
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }
}
