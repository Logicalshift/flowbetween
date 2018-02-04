use super::super::style::*;
use super::super::viewmodel::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;

use std::sync::*;
use std::time::Duration;

/// Width of an item in a virtualised canvas
const VIRTUAL_WIDTH: f32    = 400.0;

/// Height of an item in a virtualised canvas
const VIRTUAL_HEIGHT: f32   = 256.0;

/// Height of the time scale control
const SCALE_HEIGHT: f32     = 24.0;

/// Length of a tick on the timeline (in pixels)
const TICK_LENGTH: f32      = 7.0;

/// Height of a 'main' tick
const TICK_MAIN_HEIGHT: f32 = 10.0;

/// Height of an 'inbetween' tick
const TICK_HEIGHT: f32      = 5.0;

/// Height of a layer in pixels
const LAYER_HEIGHT: f32     = 24.0;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController<Anim: Animation> {
    /// The view model for this controller
    _anim_view_model:   AnimationViewModel<Anim>,

    /// The UI view model
    view_model:         Arc<DynamicViewModel>,

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
        let create_keyframe_canvas  = Self::create_draw_keyframes_fn(anim_view_model.timeline());
        let virtual_keyframes       = VirtualCanvas::new(Arc::clone(&canvases), move |x, y| (create_keyframe_canvas)(x, y));

        // Viewmodel specifies a few dynamic things
        let view_model = DynamicViewModel::new();

        view_model.set_property("IndicatorXPos", PropertyValue::Float((TICK_LENGTH*4.0) as f64));

        // UI
        let duration        = BindRef::new(&anim_view_model.timeline().duration);
        let frame_duration  = BindRef::new(&anim_view_model.timeline().frame_duration);
        let layers          = BindRef::new(&anim_view_model.timeline().layers);

        let virtual_scale_control       = virtual_scale.control();
        let virtual_keyframes_control   = virtual_keyframes.control();

        let ui = Self::ui(layers, duration, frame_duration, virtual_scale_control, virtual_keyframes_control);

        // Piece it together
        TimelineController {
            _anim_view_model:   anim_view_model,
            ui:                 ui,
            virtual_scale:      virtual_scale,
            virtual_keyframes:  virtual_keyframes,
            canvases:           canvases,
            view_model:         Arc::new(view_model)
        }
    }

    ///
    /// Creates the user interface for the timeline
    /// 
    fn ui(layers: BindRef<Vec<LayerViewModel>>, duration: BindRef<Duration>, frame_duration: BindRef<Duration>, virtual_scale_control: BindRef<Control>, virtual_keyframes_control: BindRef<Control>) -> BindRef<Control> {
        BindRef::new(&computed(move || {
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
                    Control::container()        // Scale
                        .with(Bounds {
                            x1: Position::At(0.0),
                            x2: Position::End,
                            y1: Position::At(0.0),
                            y2: Position::At(SCALE_HEIGHT)
                        })
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
                    Control::empty()            // Selected frame indicator (upper part, arrow indicator)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 16.0),
                            y1: Position::Start,
                            y2: Position::At(SCALE_HEIGHT)
                        })
                        .with(Appearance::Background(Color::Rgba(0.6, 0.4, 0.4, 0.5)))
                        .with(Scroll::Fix(FixedAxis::Vertical))
                        .with(ControlAttribute::ZIndex(4)),
                    Control::empty()            // Selected frame indicator (lower part, under the timeline)
                        .with(Bounds {
                            x1: Position::Floating(Property::Bind("IndicatorXPos".to_string()), -16.0),
                            x2: Position::Floating(Property::Bind("IndicatorXPos".to_string()), 16.0),
                            y1: Position::At(SCALE_HEIGHT),
                            y2: Position::End
                        })
                        .with(Appearance::Background(Color::Rgba(0.4, 0.4, 0.6, 0.5)))
                        .with(ControlAttribute::ZIndex(1))
                ])
                .with((ActionTrigger::VirtualScroll(VIRTUAL_WIDTH, VIRTUAL_HEIGHT), "Scroll"))
        }))
    }

    ///
    /// Creates the function for drawing the keyframes
    /// 
    fn create_draw_keyframes_fn(timeline: &TimelineViewModel<Anim>) -> Box<Fn(f32, f32) -> Box<Fn(&mut GraphicsPrimitives) -> ()+Send+Sync>+Send+Sync> {
        let timeline    = timeline.clone();

        Box::new(move |x, y| {
            // Get the layers that we'll draw
            let first_layer = (y/VIRTUAL_HEIGHT).floor() as u32;
            let last_layer  = ((y+VIRTUAL_HEIGHT)/VIRTUAL_HEIGHT).ceil() as u32 + 1;

            // ... and the keyframes in this time region
            let start_tick  = (x/TICK_LENGTH).floor() as u32;
            let end_tick    = ((x+VIRTUAL_WIDTH)/TICK_LENGTH).ceil() as u32 + 1;
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
                let end_y = (last_layer as f32) * LAYER_HEIGHT;

                gc.stroke_color(TIMESCALE_CELL);

                gc.new_path();
                for cell_index in start_tick..end_tick {
                    let cell_x = (cell_index as f32) * TICK_LENGTH;
                    let cell_x = cell_x + TICK_LENGTH/2.0;

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
                            let ypos = (layer_index as f32) * LAYER_HEIGHT;

                            // Draw the frame marker
                            gc.new_path();
                            gc.circle(xpos + TICK_LENGTH/2.0, ypos + LAYER_HEIGHT/2.0, TICK_LENGTH/2.0 - 0.5);
                            gc.fill();
                        }
                    }
                }
            })
        })
    }

    ///
    /// Draws the timeline scale
    /// 
    fn draw_scale(x: f32, _y: f32) -> Box<Fn(&mut GraphicsPrimitives) -> ()+Send+Sync> {
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
        })
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
                self.virtual_scale.virtual_scroll((VIRTUAL_WIDTH, SCALE_HEIGHT), (virtual_x, 0), (width+2, 1));
                self.virtual_keyframes.virtual_scroll((VIRTUAL_WIDTH, VIRTUAL_HEIGHT), (virtual_x, y), (width+2, height));
            },

            _ => ()
        }
    }

    fn get_viewmodel(&self) -> Option<Arc<ViewModel>> {
        Some(self.view_model.clone())
    }
}
