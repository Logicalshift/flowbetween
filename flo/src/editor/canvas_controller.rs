use super::super::viewmodel::*;
use super::super::tools::*;

use ui::*;
use canvas::*;
use desync::*;
use binding::*;
use animation::*;

use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

const MAIN_CANVAS: &str     = "main";
const PAINT_ACTION: &str    = "Paint";

///
/// Represents a layer in the current frame
/// 
struct FrameLayer {
    /// The ID of the layer to draw on the canvas
    layer_id:       u32,

    /// The frame data for this layer
    layer_frame:    Arc<Frame>
}

///
/// The core of the canvas
/// 
struct CanvasCore {
    /// The layers in the current frame
    frame_layers: HashMap<u64, FrameLayer>
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
                frame_layers: HashMap::new()
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
        self.clear_canvas(&canvas);
        canvas
    }

    ///
    /// Clears a canvas and sets it up for rendering
    /// 
    fn clear_canvas(&self, canvas: &Resource<BindingCanvas>) {
        let (width, height) = self.anim_view_model.animation().size();

        canvas.draw(move |gc| {
            gc.clear_canvas();
            gc.canvas_height((height*1.05) as f32);
            gc.center_region(0.0,0.0, width as f32, height as f32);
        });
    }

    ///
    /// Create the canvas for this controller
    ///
    fn create_main_canvas(&self) -> Resource<BindingCanvas> {
        let canvas          = self.create_canvas();
        self.canvases.assign_name(&canvas, MAIN_CANVAS);

        canvas.draw(move |gc| self.draw_background(gc));

        canvas
    }

    ///
    /// Draws the canvas background to a context
    /// 
    fn draw_background(&self, gc: &mut GraphicsPrimitives) {
        // Work out the width, height to draw the animation to draw
        let (width, height) = self.anim_view_model.animation().size();
        let (width, height) = (width as f32, height as f32);
        
        // Background always goes on layer 0
        gc.layer(0);

        gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        gc.line_width_pixels(1.0);

        // Draw the shadow
        let offset = height * 0.015;

        gc.fill_color(Color::Rgba(0.1, 0.1, 0.1, 0.4));
        gc.new_path();
        gc.rect(0.0, 0.0-offset, width+offset, height);
        gc.fill();

        // Draw the canvas background
        gc.fill_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));
        gc.new_path();
        gc.rect(0.0, 0.0, width, height);
        gc.fill();
        gc.stroke();
    }

    ///
    /// Computes the frames for all the layers in the animation
    /// 
    fn update_layers_to_frame_at_time(&self, time: Duration) {
        // Get the animation for the update
        let animation = self.anim_view_model.animation_ref();

        // Update the layers in the core
        self.core.async(move |core| {
            // Open the animation layers
            let animation   = &*animation;
            let layers      = animation.get_layer_ids();

            // Generate the frame for each layer and assign an ID
            core.frame_layers.clear();

            let mut next_layer_id = 1;

            for layer_id in layers {
                if let Some(layer) = animation.get_layer_with_id(layer_id) {
                    // Create the frame for this layer
                    let layer_frame = layer.get_frame_at_time(time);
                    
                    // Assign a layer ID to this frame and store
                    core.frame_layers.insert(layer.id(), FrameLayer {
                        layer_id:       next_layer_id,
                        layer_frame:    layer_frame
                    });

                    next_layer_id += 1;
                }
            }
        });
    }

    ///
    /// Draws the current set of frame layers
    /// 
    fn draw_frame_layers(&self) {
        let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

        // Clear the canvas and redraw the background
        self.clear_canvas(&canvas);
        canvas.draw(|gc| self.draw_background(gc));

        // Draw the active set of layers
        self.core.sync(move |core| {
            canvas.draw(move |gc| {
                // Draw the layers
                for layer in core.frame_layers.values() {
                    gc.layer(layer.layer_id);
                    layer.layer_frame.render_to(gc);
                }
            });
        });
    }

    ///
    /// Retrieves the currently selected layer
    ///
    fn get_selected_layer<'a>(&'a self) -> Option<Reader<'a, Layer>> {
        let animation = self.anim_view_model.animation_ref();

        // Find the selected layer
        let selected_layer_id = self.anim_view_model.timeline().selected_layer.get();

        // Either use the currently selected layer, or try to selecte done
        let selected_layer = match selected_layer_id {
            Some(selected_layer_id) => {
                animation.get_layer_with_id(selected_layer_id)
            },

            None => {
                // Use the first layer
                let first_layer_id = animation.get_layer_ids()
                    .into_iter()
                    .nth(0);
                
                // Mark it as the selected layer
                self.anim_view_model.timeline().selected_layer.clone().set(first_layer_id);

                // Fetch the layer from the animation
                first_layer_id.and_then(|first_layer_id| animation.get_layer_with_id(first_layer_id))
            }
        };

        selected_layer
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
            // Create the tool model for this action
            let canvas              = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
            let canvas_layer_id     = self.core.sync(|core| core.frame_layers.get(&selected_layer_id).map(|layer| layer.layer_id));
            let canvas_layer_id     = canvas_layer_id.unwrap_or(1);

            let tool_model = ToolModel {
                current_time:       self.anim_view_model.timeline().current_time.get(),
                canvas:             &canvas,
                anim_view_model:    &self.anim_view_model,
                selected_layer_id:  selected_layer_id,
                canvas_layer_id:    canvas_layer_id
            };

            // Pass the action on to the current tool
            self.anim_view_model.tools().activate_tool(&tool_model);
            effective_tool.paint(&tool_model, device, actions);
        }
    }
}

impl<Anim: Animation+'static> Controller for CanvasController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
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
