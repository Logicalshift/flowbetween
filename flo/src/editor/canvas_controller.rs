use super::super::viewmodel::*;
use super::super::tools::*;
use super::super::animation_canvas::*;

use ui::*;
use desync::*;
use binding::*;
use animation::*;

use std::sync::*;
use std::time::Duration;

const MAIN_CANVAS: &str     = "main";
const PAINT_ACTION: &str    = "Paint";

///
/// The core of the canvas
/// 
struct CanvasCore<Anim: Animation> {
    /// The canvas renderer
    renderer: CanvasRenderer,

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
pub struct CanvasController<Anim: Animation> {
    ui:                 Binding<Control>,
    canvases:           Arc<ResourceManager<BindingCanvas>>,
    anim_view_model:    AnimationViewModel<Anim>,

    core:               Desync<CanvasCore<Anim>>
}

impl<Anim: Animation+'static> CanvasController<Anim> {
    pub fn new(view_model: &AnimationViewModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases        = ResourceManager::new();

        let renderer        = CanvasRenderer::new();
        let canvas_tools    = CanvasTools::from_model(view_model);

        // Create the controller
        let mut controller = CanvasController {
            ui:                 bind(Control::empty()),
            canvases:           Arc::new(canvases),
            anim_view_model:    view_model.clone(),

            core:               Desync::new(CanvasCore {
                renderer:           renderer,
                canvas_tools:       canvas_tools,
                last_paint_device:  None,
                current_time:       Duration::new(0, 0)
            })
        };

        // The main canvas is where the current frame is rendered
        let main_canvas = controller.create_main_canvas();

        // UI is just the canvas
        controller.ui.set(Control::canvas()
            .with(main_canvas)
            .with(Bounds::fill_all())
            .with((
                (ActionTrigger::Paint(PaintDevice::Pen),                        PAINT_ACTION),
                (ActionTrigger::Paint(PaintDevice::Touch),                      PAINT_ACTION),
                (ActionTrigger::Paint(PaintDevice::Other),                      PAINT_ACTION),
                (ActionTrigger::Paint(PaintDevice::Eraser),                     PAINT_ACTION),
                (ActionTrigger::Paint(PaintDevice::Mouse(MouseButton::Left)),   PAINT_ACTION)
            )));

        // Load the initial set of frame layers
        controller.update_layers_to_frame_at_time(view_model.timeline().current_time.get());
        controller.draw_frame_layers();

        controller
    }

    ///
    /// Creates a canvas for this object
    /// 
    fn create_canvas(&self) -> Resource<BindingCanvas> {
        let canvas = self.canvases.register(BindingCanvas::new());
        canvas
    }

    ///
    /// Create the canvas for this controller
    ///
    fn create_main_canvas(&self) -> Resource<BindingCanvas> {
        let canvas          = self.create_canvas();
        self.canvases.assign_name(&canvas, MAIN_CANVAS);

        canvas
    }

    ///
    /// Computes the frames for all the layers in the animation
    /// 
    fn update_layers_to_frame_at_time(&self, time: Duration) {
        // Get the animation for the update
        let animation = self.anim_view_model.clone();

        // Update the layers in the core
        self.core.async(move |core| {
            // Update the time set in the core
            core.current_time = time;

            // Clear any existing canvases
            core.renderer.clear();

            // Open the animation layers
            let layers      = animation.get_layer_ids();

            // Load the frames into the renderer
            for layer_id in layers {
                if let Some(layer) = animation.get_layer_with_id(layer_id) {
                    core.renderer.load_frame(&*layer, time);
                }
            }
        });
    }

    ///
    /// Draws the current set of frame layers
    /// 
    fn draw_frame_layers(&self) {
        let canvas  = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
        let size    = self.anim_view_model.size();

        // Draw the active set of layers
        self.core.sync(move |core| {
            core.renderer.draw_frame_layers(&*canvas, size);
        });
    }

    ///
    /// Performs a series of painting actions on the canvas
    /// 
    fn paint(&self, device: &PaintDevice, actions: &Vec<Painting>) {
        let device = *device;

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

impl<Anim: Animation+'static> Controller for CanvasController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn tick(&self) {
        // Check that the frame time hasn't changed
        let displayed_time  = self.core.sync(|core| core.current_time);
        let target_time     = self.anim_view_model.timeline().current_time.get();

        if displayed_time != target_time {
            // If the selected frame has changed, regenerate the canvas
            self.update_layers_to_frame_at_time(target_time);
            self.draw_frame_layers();
        }
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            (PAINT_ACTION, &Paint(ref device, ref painting))    => self.paint(device, painting),
            _                                                   => ()
        };
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        Some(self.canvases.clone())
    }
}
