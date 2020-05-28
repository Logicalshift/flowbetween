use super::super::model::*;
use super::super::tools::*;
use super::super::animation_canvas::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use ::desync::*;
use futures::future;

use std::sync::*;
use std::time::Duration;

const MAIN_CANVAS: &str     = "main";
const PAINT_ACTION: &str    = "Paint";

///
/// The core of the canvas
///
struct CanvasCore<Anim: Animation+EditableAnimation> {
    /// The canvas renderer
    renderer: CanvasRenderer,

    /// The canvas invalidation count specified in the model when the canvas was drawn
    current_invalidation_count: u64,

    /// Executes actions for the canvas tools
    canvas_tools: CanvasTools<Anim>,

    /// The most recent paint device that received input for this canvas
    last_paint_device: Option<PaintDevice>,

    /// The time of the current frame
    current_time: Duration
}

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController<Anim: Animation+EditableAnimation> {
    ui:                 BindRef<Control>,
    canvases:           Arc<ResourceManager<BindingCanvas>>,
    anim_model:         FloModel<Anim>,
    tool_changed:       Arc<Mutex<bool>>,
    _onion_skin_model:  BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)>,

    core:               Arc<Desync<CanvasCore<Anim>>>
}

impl<Anim: Animation+EditableAnimation+'static> CanvasController<Anim> {
    ///
    /// Creates a new canvas controller
    ///
    pub fn new(view_model: &FloModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases            = ResourceManager::new();

        let renderer            = CanvasRenderer::new();
        let canvas_tools        = CanvasTools::from_model(view_model);
        let main_canvas         = Self::create_main_canvas(&canvases);
        let ui                  = Self::ui(main_canvas.clone(), view_model.size.clone());
        let tool_changed        = Arc::new(Mutex::new(true));
        let onion_skin_model    = Self::onion_skin_binding(view_model);

        // Set the tool changed flag whenever the effective tool changes
        // Note: the keep_alive() here will leak if the controller lives for less time than the model
        let also_tool_changed = Arc::clone(&tool_changed);
        view_model.tools().effective_tool
            .when_changed(notify(move || { *also_tool_changed.lock().unwrap() = true; }))
            .keep_alive();

        // Create the core to perform the actual rendering
        let core                = Desync::new(CanvasCore {
                renderer:                   renderer,
                canvas_tools:               canvas_tools,
                last_paint_device:          None,
                current_time:               Duration::new(0, 0),
                current_invalidation_count: 0
            });
        let core                = Arc::new(core);

        // Connect events to the core
        Self::pipe_onion_skin_renders(main_canvas.clone(), onion_skin_model.clone(), core.clone());

        // Create the controller
        let controller = CanvasController {
            ui:                 ui,
            canvases:           Arc::new(canvases),
            anim_model:         view_model.clone(),
            tool_changed:       tool_changed,
            _onion_skin_model:  onion_skin_model,

            core:               core
        };

        // Load the initial set of frame layers
        controller.update_layers_to_frame_at_time(view_model.timeline().current_time.get());
        controller.draw_frame_layers();

        controller
    }

    ///
    /// Creates a binding from a model to the parameters of the onion skin renderer function
    ///
    fn onion_skin_binding(view_model: &FloModel<Anim>) -> BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)> {
        let onion_skin_model    = view_model.onion_skin();
        let past_color          = onion_skin_model.past_color.clone();
        let future_color        = onion_skin_model.future_color.clone();
        let onion_skins         = onion_skin_model.onion_skins.clone();

        BindRef::from(computed(move || {
            (past_color.get(), future_color.get(), onion_skins.get())
        }))
    }

    ///
    /// Updates the rendering in the core whenever the onion skins change
    ///
    fn pipe_onion_skin_renders(canvas: Resource<BindingCanvas>, binding: BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)>, core: Arc<Desync<CanvasCore<Anim>>>) {
        let onion_skin_stream   = follow(binding);
        let renderer            = OnionSkinRenderer::new();

        pipe_in(core, onion_skin_stream, move |core, (past_color, future_color, onion_skins)| {
            renderer.render(&*canvas, &mut core.renderer, onion_skins, past_color, future_color);
            Box::pin(future::ready(()))
        })
    }

    ///
    /// Creates the ui for the canvas controller
    ///
    fn ui(main_canvas: Resource<BindingCanvas>, size: BindRef<(f64, f64)>) -> BindRef<Control> {
        let ui = computed(move || {
            let main_canvas     = main_canvas.clone();
            let size            = size.get();
            let (width, height) = size;
            let (width, height) = (width as f32, height as f32);

            Control::scrolling_container()
                .with(Bounds::fill_all())
                .with(Scroll::MinimumContentSize(width, height))
                .with(vec![
                    Control::canvas()
                        .with(main_canvas)
                        .with(Bounds::fill_all())
                        .with(Hint::FastDrawing)
                        .with((
                            (ActionTrigger::Paint(PaintDevice::Pen),                        PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Other),                      PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Eraser),                     PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Mouse(MouseButton::Left)),   PAINT_ACTION)
                        ))
                ])
        });

        BindRef::from(ui)
    }

    ///
    /// Create the canvas for this controller
    ///
    fn create_main_canvas(resources: &ResourceManager<BindingCanvas>) -> Resource<BindingCanvas> {
        let canvas          = resources.register(BindingCanvas::new());
        resources.assign_name(&canvas, MAIN_CANVAS);

        canvas
    }

    ///
    /// Computes the frames for all the layers in the animation
    ///
    fn update_layers_to_frame_at_time(&self, time: Duration) {
        // Retrieve the layers from the animation
        let layers              = self.anim_model.frame().layers.get();
        let invalidate_count    = self.anim_model.timeline().canvas_invalidation_count.get();

        // Update the layers in the core
        self.core.desync(move |core| {
            // Update the time set in the core
            core.current_time               = time;
            core.current_invalidation_count = invalidate_count;

            // Clear any existing canvases
            core.renderer.clear();

            // Load the frames into the renderer
            for layer_frame in layers {
                core.renderer.load_frame(layer_frame);
            }
        });
    }

    ///
    /// Draws the current set of frame layers
    ///
    fn draw_frame_layers(&self) {
        let canvas  = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
        let size    = self.anim_model.size();

        // Draw the active set of layers
        self.core.sync(move |core| {
            core.renderer.draw_frame_layers(&*canvas, size);
            core.renderer.draw_overlays(&*canvas);
        });
    }

    ///
    /// Performs a series of painting actions on the canvas
    ///
    fn paint(&self, device: &PaintDevice, actions: &Vec<Painting>) {
        let device = *device;

        // Update the paint device in the tool model if we're starting a new paint action
        if actions.len() > 0 && actions[0].action == PaintAction::Start {
            self.anim_model.tools().current_pointer.set((device, actions[0].pointer_id));
        }

        // Fetch the canvas we're going to draw to
        let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

        // Convert the actions into tool inputs
        let tool_inputs = actions.iter()
            .map(|painting| ToolInput::Paint(painting.clone()));

        // Send to the canvas tools object
        self.core.sync(move |core| {
            let mut extra_inputs = vec![];

            // If the paint device has changed, then send a tool input indicating that that has occurred
            if Some(device) != core.last_paint_device {
                core.last_paint_device = Some(device);
                extra_inputs.push(ToolInput::PaintDevice(device));
            }

            // Amend the inputs
            let tool_inputs = extra_inputs.into_iter().chain(tool_inputs);

            // Send the inputs
            core.canvas_tools.send_input(&canvas, &mut core.renderer, tool_inputs)
        });
    }
}

impl<Anim: Animation+EditableAnimation+'static> Controller for CanvasController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn tick(&self) {
        // Ensure that the active tool is up to date
        if *self.tool_changed.lock().unwrap() {
            // Tool has changed: need to call refresh()
            self.core.sync(|core| {
                // Fetch the canvas to deal with the refresh
                let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

                // Tool has no longer changed
                *self.tool_changed.lock().unwrap() = false;

                // Refresh the tool (this will update any overlays, for example)
                core.canvas_tools.refresh_tool(&*canvas, &mut core.renderer);
            });
        } else {
            // Process any pending actions for the current tool
            self.core.sync(|core| {
                let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
                core.canvas_tools.poll_for_pending_actions(&*canvas, &mut core.renderer)
            });
        }

        // Check that the frame time hasn't changed and the frame has not been invalidated since it was last drawn
        let displayed_invalidation_count    = self.core.sync(|core| core.current_invalidation_count);
        let displayed_time                  = self.core.sync(|core| core.current_time);
        let target_invalidation_count       = self.anim_model.timeline().canvas_invalidation_count.get();
        let target_time                     = self.anim_model.timeline().current_time.get();

        if displayed_time != target_time || displayed_invalidation_count != target_invalidation_count {
            // If the selected frame has changed, regenerate the canvas
            self.update_layers_to_frame_at_time(target_time);
            self.draw_frame_layers();
        }
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use self::ActionParameter::*;

        match (action_id, action_parameter) {
            (PAINT_ACTION, &Paint(ref device, ref painting))    => self.paint(device, painting),
            _                                                   => ()
        };
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        Some(self.canvases.clone())
    }
}
