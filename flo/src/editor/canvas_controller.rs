use super::super::viewmodel::*;
use super::super::tools::*;
use super::super::animation_canvas::*;

use ui::*;
use desync::*;
use binding::*;
use animation::*;

use typemap::*;
use std::sync::*;
use std::time::Duration;

const MAIN_CANVAS: &str     = "main";
const PAINT_ACTION: &str    = "Paint";

///
/// The core of the canvas
/// 
struct CanvasCore {
    /// The canvas renderer
    renderer: CanvasRenderer,

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

    core:               Desync<CanvasCore>
}

impl<Anim: Animation+'static> CanvasController<Anim> {
    pub fn new(view_model: &AnimationViewModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases = ResourceManager::new();

        // Create the controller
        let mut controller = CanvasController {
            ui:                 bind(Control::empty()),
            canvases:           Arc::new(canvases),
            anim_view_model:    view_model.clone(),

            core:               Desync::new(CanvasCore {
                renderer:       CanvasRenderer::new(),
                current_time:   Duration::new(0, 0)
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
        // Set the current pointer
        let pointer_id = actions.first().map(|first_action| first_action.pointer_id).unwrap_or(0);
        self.anim_view_model.tools().current_pointer.clone().set((*device, pointer_id));

        // Get the active tool
        let effective_tool = self.anim_view_model.tools().effective_tool.get();

        // Get the selected layer
        let selected_layer_id = self.anim_view_model.timeline().selected_layer.get();

        if let (Some(selected_layer_id), Some(effective_tool)) = (selected_layer_id, effective_tool) {
            /*
            // Create the tool model for this action
            let canvas              = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
            let canvas_layer_id     = self.core.sync(|core| core.frame_layers.get(&selected_layer_id).map(|layer| layer.layer_id));
            let canvas_layer_id     = canvas_layer_id.unwrap_or(1);

            let tool_model = ToolModel {
                current_time:       self.anim_view_model.timeline().current_time.get(),
                canvas:             &canvas,
                anim_view_model:    &self.anim_view_model,
                selected_layer_id:  selected_layer_id,
                canvas_layer_id:    canvas_layer_id,
                tool_state:         self.tool_state.clone()
            };

            // Pass the action on to the current tool
            self.anim_view_model.tools().activate_tool(&tool_model);
            // effective_tool.paint(&tool_model, device, actions);
            */
        }
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
