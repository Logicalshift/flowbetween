use super::timeline_layer_list_controller::*;
use super::timeline_layer_controls_controller::*;
use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

/// Side of the onion skin indicator
enum Onion {
    Left,
    Right
}

/// Action when the user drags the timeline 'time' indicator
const DRAG_TIMELINE_POSITION: &str = "DragTime";

/// Action when the user drags the timeline 'future onion skins' indicator
const DRAG_ONION_FRAMES_BEFORE: &str = "DragOnionBefore";

/// Action when the user drags the timeline 'past onion skins' indicator
const DRAG_ONION_FRAMES_AFTER: &str = "DragOnionAfter";

/// Action when the user clicks/drags on the scale away from the 'time' indicator
const CLICK_AND_DRAG_TIMELINE_POSITION: &str = "ClickTime";

/// Action when the virtual scroll position changes
const SCROLL_TIMELINE: &str     = "Scroll";

/// Width of an item in a virtualised canvas
const VIRTUAL_WIDTH: f32        = 400.0;

/// Height of an item in a virtualised canvas
const VIRTUAL_HEIGHT: f32       = 256.0;

/// Height of the time scale control
pub const TIMELINE_SCALE_HEIGHT: f32         = 24.0;

/// Length of a tick on the timeline (in pixels)
const TICK_LENGTH: f32          = 7.0;

/// Height of a 'main' tick
const TICK_MAIN_HEIGHT: f32     = 10.0;

/// Height of an 'inbetween' tick
const TICK_HEIGHT: f32          = 5.0;

/// Height of a layer in pixels
pub const TIMELINE_LAYER_HEIGHT: f32         = 24.0;

/// Width of the layer name panel
const LAYER_PANEL_WIDTH: f32    = 256.0;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController<Anim: Animation> {
    /// The view model for this controller
    anim_model:         FloModel<Anim>,

    /// The UI view model
    view_model:                 Arc<DynamicViewModel>,

    /// The canvases for the timeline
    canvases:                   Arc<ResourceManager<BindingCanvas>>,

    /// The current_time at the most recent drag start position
    drag_start_time:            Binding<Duration>,

    /// The setting of frames_before/frames_after when the drag on the onion skin start/end indicators started
    drag_start_frames:          Binding<usize>,

    /// A virtual control that draws the timeline scale
    virtual_scale:              VirtualCanvas,

    /// A virtual control that draws the keyframes
    virtual_keyframes:          VirtualCanvas,

    /// A controller to display the UI for managing the layers
    layer_list_controller:      Arc<TimelineLayerListController>,

    /// A controller to display the UI for adding/removing layers
    layer_controls_controller:  Arc<TimelineLayerControlsController<Anim>>,

    /// The UI for the timeline
    ui:                         BindRef<Control>
}

impl<Anim: 'static+Animation+EditableAnimation> TimelineController<Anim> {
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
        let current_time        = anim_model.timeline().current_time.clone();
        let frame_duration      = anim_model.timeline().frame_duration.clone();
        let frames_before       = anim_model.onion_skin().frames_before.clone();
        let frames_after        = anim_model.onion_skin().frames_after.clone();

        let indicator_x_pos = computed(move || {
            let current_time        = current_time.get();
            let frame_duration      = frame_duration.get();

            let current_time_ns     = current_time.as_secs() * 1_000_000_000 + (current_time.subsec_nanos() as u64);
            let frame_duration_ns   = frame_duration.as_secs() * 1_000_000_000 + (frame_duration.subsec_nanos() as u64);

            let frame               = (current_time_ns+(frame_duration_ns/2)) / frame_duration_ns;

            let tick_length         = TICK_LENGTH as f64;
            let tick_x              = (frame as f64) * tick_length;
            let tick_x              = tick_x + (tick_length/2.0);
            let tick_x              = tick_x + (LAYER_PANEL_WIDTH as f64);

            tick_x
        });

        let xpos                = indicator_x_pos.clone();
        let indicator_left_pos  = computed(move || {
            let xpos            = xpos.get();
            let tick_length     = TICK_LENGTH as f64;
            let frames_before   = frames_before.get() as f64;
            let left_distance   = frames_before * tick_length;

            xpos - left_distance
        });
        let xpos                = indicator_x_pos.clone();
        let indicator_right_pos  = computed(move || {
            let xpos            = xpos.get();
            let tick_length     = TICK_LENGTH as f64;
            let frames_after    = frames_after.get() as f64;
            let right_distance  = frames_after * tick_length;

            xpos + right_distance
        });


        // Indicator view model
        view_model.set_computed("IndicatorXPos", move || PropertyValue::Float(indicator_x_pos.get()));
        view_model.set_computed("IndicatorLeft", move || PropertyValue::Float(indicator_left_pos.get()));
        view_model.set_computed("IndicatorRight", move || PropertyValue::Float(indicator_right_pos.get()));

        // UI
        let layer_list_controller       = TimelineLayerListController::new(&anim_model);
        let layer_controls_controller   = TimelineLayerControlsController::new(&anim_model);

        let duration                    = BindRef::new(&anim_model.timeline().duration);
        let frame_duration              = BindRef::new(&anim_model.timeline().frame_duration);
        let layers                      = BindRef::new(&anim_model.timeline().layers);

        let virtual_scale_control       = virtual_scale.control();
        let virtual_keyframes_control   = virtual_keyframes.control();

        let ui = Self::ui(layers, duration, frame_duration, virtual_scale_control, virtual_keyframes_control, Arc::clone(&canvases), anim_model.onion_skin());

        // Piece it together
        TimelineController {
            anim_model:                 anim_model,
            ui:                         ui,
            virtual_scale:              virtual_scale,
            virtual_keyframes:          virtual_keyframes,
            drag_start_time:            bind(Duration::from_millis(0)),
            drag_start_frames:          bind(0),
            canvases:                   canvases,
            layer_list_controller:      Arc::new(layer_list_controller),
            layer_controls_controller:  Arc::new(layer_controls_controller),
            view_model:                 Arc::new(view_model)
        }
    }

    ///
    /// Creates the user interface for the timeline
    ///
    fn ui(layers: BindRef<Vec<LayerModel>>, duration: BindRef<Duration>, frame_duration: BindRef<Duration>, virtual_scale_control: BindRef<Control>, virtual_keyframes_control: BindRef<Control>, canvases: Arc<ResourceManager<BindingCanvas>>, onion_skin: &OnionSkinModel<Anim>) -> BindRef<Control> {
        let timescale_indicator         = BindingCanvas::with_drawing(Self::draw_frame_indicator);
        let timescale_indicator         = canvases.register(timescale_indicator);

        let left_onion_indicator        = BindingCanvas::with_drawing(|gc| Self::draw_onion_indicator(gc, Onion::Left));
        let left_onion_indicator        = canvases.register(left_onion_indicator);
        let right_onion_indicator       = BindingCanvas::with_drawing(|gc| Self::draw_onion_indicator(gc, Onion::Right));
        let right_onion_indicator       = canvases.register(right_onion_indicator);

        let timescale_indicator_line    = BindingCanvas::with_drawing(Self::draw_frame_indicator_line);
        let timescale_indicator_line    = canvases.register(timescale_indicator_line);

        let show_onion_skins            = onion_skin.show_onion_skins.clone();
        let frames_before               = onion_skin.frames_before.clone();
        let frames_after                = onion_skin.frames_after.clone();

        BindRef::new(&computed(move || {
            let timescale_indicator         = timescale_indicator.clone();
            let timescale_indicator_line    = timescale_indicator_line.clone();

            // Work out the number of frames in this animation
            let duration                = duration.get();
            let frame_duration          = frame_duration.get();
            let layers                  = layers.get();

            let duration_ns             = duration.as_secs()*1_000_000_000 + (duration.subsec_nanos() as u64);
            let frame_duration_ns       = frame_duration.as_secs()*1_000_000_000 + (frame_duration.subsec_nanos() as u64);

            let width                   = TICK_LENGTH * ((duration_ns / frame_duration_ns) as f32);
            let height                  = (layers.len() as f32) * TIMELINE_LAYER_HEIGHT;

            // If the user has enabled the onion skin display, then indicate the region that they are covering
            let onion_skin_indicators   = if show_onion_skins.get() {
                let frames_before   = frames_before.get() as f32;
                let frames_after    = frames_after.get() as f32;

                vec![
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_ONION_INDICATOR))
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -frames_before * TICK_LENGTH),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), frames_after * TICK_LENGTH),
                            y1: Position::At(3.0),
                            y2: Position::At(4.0)
                        })
                    .with(ControlAttribute::ZIndex(3)),
                    Control::canvas()
                        .with(left_onion_indicator.clone())
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorLeft".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorLeft".to_string()), 16.0),
                            y1: Position::At(0.0),
                            y2: Position::At(TIMELINE_SCALE_HEIGHT)
                        })
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with((ActionTrigger::Drag, DRAG_ONION_FRAMES_BEFORE))
                        .with(ControlAttribute::ZIndex(3)),
                    Control::canvas()
                        .with(right_onion_indicator.clone())
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorRight".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorRight".to_string()), 16.0),
                            y1: Position::At(0.0),
                            y2: Position::At(TIMELINE_SCALE_HEIGHT)
                        })
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with((ActionTrigger::Drag, DRAG_ONION_FRAMES_AFTER))
                        .with(ControlAttribute::ZIndex(3)),
                ]
            } else {
                vec![]
            };

            // Build the final control
            Control::scrolling_container()
                .with(Bounds::fill_all())
                .with(Scroll::MinimumContentSize(width, height + TIMELINE_SCALE_HEIGHT))
                .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Always))
                .with(Scroll::VerticalScrollBar(ScrollBarVisibility::OnlyIfNeeded))
                .with(Appearance::Background(TIMELINE_BACKGROUND))
                .with(vec![
                    Control::container()        // Layer controls
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::At(LAYER_PANEL_WIDTH),
                            y1: Position::At(0.0),
                            y2: Position::At(TIMELINE_SCALE_HEIGHT)
                        })
                        .with(vec![
                            Control::container()
                                .with(Bounds::stretch_horiz(1.0))
                                .with_controller("LayerControls"),
                            Control::empty()
                                .with(Bounds::next_horiz(1.0))
                                .with(Appearance::Background(TIMESCALE_BORDER))
                        ])
                        .with(Appearance::Background(TIMESCALE_BACKGROUND))
                        .with(Scroll::Fix(FixedAxis::Both))
                        .with(ControlAttribute::ZIndex(6)),
                    Control::container()        // Layer list
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::At(LAYER_PANEL_WIDTH),
                            y1: Position::At(TIMELINE_SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(vec![
                            Control::container()
                                .with(Bounds::stretch_horiz(1.0))
                                .with_controller("LayerList"),
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
                            y2: Position::At(TIMELINE_SCALE_HEIGHT)
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
                            y1: Position::At(TIMELINE_SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(vec![
                            virtual_keyframes_control.get()
                        ])
                        .with(ControlAttribute::ZIndex(2)),
                    Control::canvas()           // Selected frame indicator (upper part, arrow indicator)
                        .with(timescale_indicator)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -6.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 6.0),
                            y1: Position::Start,
                            y2: Position::At(TIMELINE_SCALE_HEIGHT)
                        })
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with(Appearance::Background(Color::Rgba(0.0, 0.0, 0.0, 0.0)))
                        .with((ActionTrigger::Drag, DRAG_TIMELINE_POSITION))
                        .with(ControlAttribute::ZIndex(4)),
                    Control::canvas()           // Selected frame indicator (lower part, under the timeline)
                        .with(timescale_indicator_line)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 16.0),
                            y1: Position::At(TIMELINE_SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(ControlAttribute::ZIndex(1))
                ].into_iter().chain(onion_skin_indicators).collect::<Vec<_>>())
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
            let first_layer = (y/TIMELINE_LAYER_HEIGHT).floor() as usize;
            let last_layer  = ((y+VIRTUAL_HEIGHT)/TIMELINE_LAYER_HEIGHT).ceil() as usize + 1;

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

                let last_layer  = last_layer.min(layers.len());
                let end_tick    = end_tick;

                let index_for_layer = layers.iter()
                    .enumerate()
                    .map(|(index, layer)| (layer.id, index))
                    .collect::<HashMap<_, _>>();

                // Center the drawing region
                gc.canvas_height(-VIRTUAL_HEIGHT);
                gc.center_region(x, y, x+VIRTUAL_WIDTH, y+VIRTUAL_HEIGHT);

                gc.line_width(0.5);

                // Draw the cell dividers
                let end_x = (end_tick as f32) * TICK_LENGTH;
                let end_x = end_x + LAYER_PANEL_WIDTH;
                let end_y = (last_layer as f32) * TIMELINE_LAYER_HEIGHT;

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
                    let layer_y = ((layer_index as f32) * TIMELINE_LAYER_HEIGHT) - 0.5;

                    gc.move_to(x, layer_y + TIMELINE_LAYER_HEIGHT);
                    gc.line_to(end_x, layer_y + TIMELINE_LAYER_HEIGHT);
                }
                gc.stroke();

                // Draw the keyframes that are in this region
                gc.fill_color(TIMESCALE_KEYFRAME);
                for keyframe in keyframes.iter() {
                    // Fetch where this frame occurs
                    let frame       = keyframe.frame;
                    let layer_id    = keyframe.layer_id;
                    let layer_index = index_for_layer.get(&layer_id);

                    // Draw it if it's in this view
                    if let Some(layer_index) = layer_index {
                        if layer_index >= &first_layer && layer_index < &last_layer {
                            // Top-left corner of this frame
                            let xpos = (frame as f32) * TICK_LENGTH;
                            let xpos = xpos + LAYER_PANEL_WIDTH;
                            let ypos = (*layer_index as f32) * TIMELINE_LAYER_HEIGHT;

                            // Draw the frame marker
                            gc.new_path();
                            gc.circle(xpos + TICK_LENGTH/2.0, ypos + TIMELINE_LAYER_HEIGHT/2.0, TICK_LENGTH/2.0 - 0.5);
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
            gc.canvas_height(TIMELINE_SCALE_HEIGHT);
            gc.center_region(x, 0.0, x+VIRTUAL_WIDTH, TIMELINE_SCALE_HEIGHT);
            gc.line_width(1.0);

            // Fill the background
            gc.fill_color(TIMESCALE_BACKGROUND);
            gc.new_path();
            gc.rect(x, 0.0, x+VIRTUAL_WIDTH, TIMELINE_SCALE_HEIGHT);
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

        gc.new_path();
        gc.move_to(-0.35, -0.1);
        gc.line_to(-0.35, 0.8);
        gc.line_to(0.35, 0.8);
        gc.line_to(0.35, -0.1);

        gc.line_to(0.0, -1.0);
        gc.line_to(-0.35, -0.1);
        gc.close_path();

        gc.stroke_color(TIMESCALE_INDICATOR_OUTER_GLOW);
        gc.line_width_pixels(1.0);
        gc.stroke();

        gc.fill_color(TIMESCALE_INDICATOR);
        gc.fill();

        gc.stroke_color(TIMESCALE_INDICATOR_INNER_BORDER);
        gc.line_width_pixels(0.5);
        gc.stroke();

        gc.stroke_color(TIMESCALE_INDICATOR_GRIP);
        gc.new_path();
        gc.move_to(-0.15, 0.05);
        gc.line_to(-0.15, 0.45);
        gc.move_to(0.15, 0.05);
        gc.line_to(0.15, 0.45);
        gc.move_to(0.0, -0.1);
        gc.line_to(0.0, 0.6);
        gc.stroke();
    }

    ///
    /// Draws the frame indicator
    ///
    fn draw_onion_indicator(gc: &mut dyn GraphicsPrimitives, side: Onion) -> () {
        gc.canvas_height(2.05);

        gc.new_path();
        match side {
            Onion::Right => {
                gc.move_to(0.0, 0.1);
                gc.line_to(0.0, 0.8);
                gc.line_to(0.6, 0.8);
                gc.line_to(0.6, 0.1);

                gc.line_to(0.0, -1.0);
                gc.line_to(0.0, 0.1);
                gc.close_path();
            },

            Onion::Left => {
                gc.move_to(-0.6, 0.1);
                gc.line_to(-0.6, 0.8);
                gc.line_to(0.0, 0.8);
                gc.line_to(0.0, 0.1);

                gc.line_to(0.0, -1.0);
                gc.line_to(-0.6, 0.1);
                gc.close_path();
            },
        }

        gc.stroke_color(TIMESCALE_ONION_INDICATOR_OUTER);
        gc.line_width_pixels(1.0);
        gc.stroke();

        gc.fill_color(TIMESCALE_ONION_INDICATOR);
        gc.fill();

        gc.stroke_color(TIMESCALE_ONION_INDICATOR_INNER);
        gc.line_width_pixels(0.5);
        gc.stroke();
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

impl<Anim: EditableAnimation+Animation+'static> Controller for TimelineController<Anim> {
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
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, TIMELINE_SCALE_HEIGHT), (virtual_x, 0), (width+2, 1));
                self.virtual_keyframes.virtual_scroll((VIRTUAL_WIDTH, VIRTUAL_HEIGHT), (virtual_x, y), (width+2, height));
            },

            (CLICK_AND_DRAG_TIMELINE_POSITION, &Drag(DragAction::Start, (start_x, _start_y), _)) => {
                // Clicking on the scale moves the time to where the user clicked initially
                let time_ns = self.xpos_to_ns(start_x - LAYER_PANEL_WIDTH - (TICK_LENGTH/2.0));
                let time    = Self::ns_to_duration(time_ns);

                self.anim_model.timeline().current_time.set(time);
                self.drag_start_time.set(time);
            },

            (DRAG_TIMELINE_POSITION, &Drag(DragAction::Start, _, _)) => {
                // Remember the start time when a drag begins
                self.drag_start_time.set(self.anim_model.timeline().current_time.get());
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
                timeline.current_time.set(new_time);
            },

            (DRAG_ONION_FRAMES_AFTER, &Drag(DragAction::Start, _, _)) => {
                self.drag_start_frames.set(self.anim_model.onion_skin().frames_after.get());
            },

            (DRAG_ONION_FRAMES_BEFORE, &Drag(DragAction::Start, _, _)) => {
                self.drag_start_frames.set(self.anim_model.onion_skin().frames_before.get());
            },

            (DRAG_ONION_FRAMES_AFTER, &Drag(_drag_type, (start_x, _start_y), (x, _y)))
            | (DRAG_ONION_FRAMES_BEFORE, &Drag(_drag_type, (start_x, _start_y), (x, _y))) => {
                // Work out the difference in frames
                let is_before       = action_id == DRAG_ONION_FRAMES_BEFORE;
                let initial_frames  = self.drag_start_frames.get();
                let mut frame_diff  = ((x - start_x)/TICK_LENGTH).floor() as i32;

                // The 'before' frames moves in the opposite direction to the 'after' frames
                if is_before { frame_diff = -frame_diff; }

                // Work out the new number of frames (with limits)
                let new_frames      = (initial_frames as i32) + frame_diff;
                let new_frames      = if new_frames <= 0 { 1 } else { new_frames };
                let new_frames      = if new_frames > 10 { 10 } else { new_frames };

                // Update the model
                if is_before {
                    self.anim_model.onion_skin().frames_before.set(new_frames as usize);
                } else {
                    self.anim_model.onion_skin().frames_after.set(new_frames as usize);
                }
            },

            _ => ()
        }
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_subcontroller(&self, controller: &str) -> Option<Arc<dyn Controller>> {
        match controller {
            "LayerList"         => Some(self.layer_list_controller.clone()),
            "LayerControls"     => Some(self.layer_controls_controller.clone()),
            _                   => None
        }
    }
}
