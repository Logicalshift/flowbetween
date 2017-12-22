use super::viewmodel::*;

use ui::*;
use ui::canvas::*;
use desync::*;
use binding::*;
use animation::*;

use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

use curves::*;
use curves::bezier;

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
    ui_view_model:      Arc<NullViewModel>,
    ui:                 Binding<Control>,
    canvases:           Arc<ResourceManager<Canvas>>,
    anim_view_model:    AnimationViewModel<Anim>,

    core:               Desync<CanvasCore>
}

impl<Anim: Animation+'static> CanvasController<Anim> {
    pub fn new(view_model: &AnimationViewModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases = ResourceManager::new();

        // Create the controller
        let mut controller = CanvasController {
            ui_view_model:      Arc::new(NullViewModel::new()),
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
    fn create_canvas(&self) -> Resource<Canvas> {
        let canvas = self.canvases.register(Canvas::new());
        self.clear_canvas(&canvas);
        canvas
    }

    ///
    /// Clears a canvas and sets it up for rendering
    /// 
    fn clear_canvas(&self, canvas: &Resource<Canvas>) {
        let (width, height) = open_read::<AnimationSize>(self.anim_view_model.animation())
            .map(|size| size.size())
            .unwrap_or((1920.0, 1080.0));

        canvas.draw(move |gc| {
            gc.clear_canvas();
            gc.canvas_height((height*1.05) as f32);
            gc.center_region(0.0,0.0, width as f32, height as f32);
        });
    }

    ///
    /// Create the canvas for this controller
    ///
    fn create_main_canvas(&self) -> Resource<Canvas> {
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
        let (width, height) = open_read::<AnimationSize>(self.anim_view_model.animation())
            .map(|size| size.size())
            .unwrap_or((1920.0, 1080.0));
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
            let layers      = open_read::<AnimationLayers>(animation);

            // Generate the frame for each layer and assign an ID
            core.frame_layers.clear();

            if let Some(layers) = layers {
                let mut next_layer_id = 1;

                for layer in layers.layers() {
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

        // TEST: code for a bezier curve
        /*
        canvas.draw(|gc| {
            let curve           = bezier::Curve::from_points(Coord2(100.0, 100.0), Coord2(300.0, 150.0), Coord2(50.0, 200.0), Coord2(250.0, 350.0));
            let offset_curve1   = bezier::offset(&curve, 10.0, 40.0);
            let offset_curve2   = bezier::offset(&curve, -10.0, -40.0);

            gc.stroke_color(Color::Rgba(0.0, 0.0, 1.0, 1.0));

            gc.new_path();
            gc.move_to(curve.start_point().x(), curve.start_point().y());
            gc_draw_bezier(gc, &curve);
            gc.stroke();

            gc.stroke_color(Color::Rgba(1.0, 0.0, 0.0, 1.0));

            gc.new_path();
            gc.move_to(offset_curve1[0].start_point().x(), offset_curve1[0].start_point().y());
            for c in offset_curve1 {
                gc_draw_bezier(gc, &c);
            }

            gc.move_to(offset_curve2[0].start_point().x(), offset_curve2[0].start_point().y());
            for c in offset_curve2 {
                gc_draw_bezier(gc, &c);
            }

            gc.stroke();
        });
        */

        /*
        // TEST: code for a bezier curve
        canvas.draw(|gc| {
            // Try fitting a curve to some points
            let fit_points = vec![
                Coord2(100.0, 800.0),
                Coord2(120.0, 760.0),
                Coord2(200.0, 790.0),
                Coord2(340.0, 820.0),
                Coord2(410.0, 840.0),
                Coord2(490.0, 830.0),
                Coord2(550.0, 830.0),
                Coord2(720.0, 860.0),
                Coord2(800.0, 880.0),
                Coord2(900.0, 800.0)
            ];

            let fit_curves = bezier::Curve::fit_from_points(&fit_points, 20.0).unwrap();

            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            gc.new_path();
            gc.move_to(fit_curves[0].start_point().x(), fit_curves[0].start_point().y());
            for fit_curve in fit_curves {
                gc_draw_bezier(gc, &fit_curve);
            }
            gc.stroke();

            gc.stroke_color(Color::Rgba(0.0, 0.5, 1.0, 1.0));
            for point in fit_points {
                gc.new_path();
                gc.rect(point.x() - 2.0, point.y() - 2.0, point.x() + 2.0, point.y() + 2.0);
                gc.stroke();
            }

            // Draw a curve, split and its bounding box
            let curve               = bezier::Curve::from_points(Coord2(100.0, 500.0), Coord2(1800.0, 400.0), Coord2(300.0,0.0), Coord2(1700.0, 1000.0));
            let (curve1, curve2)    = curve.subdivide(0.33);
            let (_curve, curve3)    = curve.subdivide(0.66);

            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            gc.new_path();
            gc.move_to(curve.start_point().x(), curve.start_point().y());
            gc_draw_bezier(gc, &curve);
            gc.stroke();

            gc.stroke_color(Color::Rgba(0.0, 0.5, 1.0, 1.0));
            gc.new_path();
            gc.move_to(curve1.start_point().x(), curve1.start_point().y());
            gc_draw_bezier(gc, &curve1);
            gc.stroke();

            gc.stroke_color(Color::Rgba(0.0, 0.7, 1.0, 1.0));
            gc.new_path();
            gc.move_to(curve3.start_point().x(), curve3.start_point().y());
            gc_draw_bezier(gc, &curve3);
            gc.stroke();

            gc.stroke_color(Color::Rgba(0.0, 0.0, 1.0, 1.0));
            let bounds = curve.bounding_box();
            gc.new_path();
            gc.rect(bounds.0.x(), bounds.0.y(), bounds.1.x(), bounds.1.y());
            gc.stroke();

            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        });
        */

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
    fn get_selected_layer(&self) -> Option<Arc<Layer>> {
        // Reading the layers from the animation
        let layers = open_read::<AnimationLayers>(self.anim_view_model.animation()).unwrap();
        
        // Find the selected layer
        let selected_layer_id = self.anim_view_model.timeline().selected_layer.get();

        // Either use the currently selected layer, or try to selecte done
        let selected_layer = match selected_layer_id {
            Some(selected_layer_id) => {
                // Use the layer with the matching ID
                layers.layers()
                    .filter(|layer| layer.id() == selected_layer_id)
                    .nth(0)
            },
            None => {
                // Use the first layer
                let first_layer = layers.layers()
                    .nth(0);
                
                // Mark it as the selected layer
                self.anim_view_model.timeline().selected_layer.clone().set(first_layer.map(|layer| layer.id()));

                first_layer
            }
        };

        selected_layer.cloned()
    }

    ///
    /// Performs a single painting action on the canvas
    /// 
    fn paint_action(&self, layer_id: u64, layer: &mut PaintLayer, action: &Painting) {
        // Get when this paint stroke is being made
        let current_time = self.anim_view_model.timeline().current_time.get();

        // Find the canvas
        let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

        // Get the canvas layer ID
        let canvas_layer_id = self.core.sync(|core| core.frame_layers.get(&layer_id).map(|layer| layer.layer_id));
        let canvas_layer_id = canvas_layer_id.unwrap_or(1);

        canvas.draw(move |gc| {
            // Perform the action
            match action.action {
                PaintAction::Start       => {
                    // Select the layer and store the current image state
                    gc.layer(canvas_layer_id);
                    gc.store();

                    // Begin the brush stroke
                    layer.start_brush_stroke(current_time, BrushPoint::from(action));
                },

                PaintAction::Continue    => {
                    // Append to the brush stroke
                    layer.continue_brush_stroke(BrushPoint::from(action));
                },

                PaintAction::Finish      => {
                    // Draw the 'final' brush stroke
                    gc.restore();
                    layer.draw_current_brush_stroke(gc);

                    // Finish the brush stroke
                    layer.finish_brush_stroke();
                },

                PaintAction::Cancel      => {
                    // Cancel the brush stroke
                    layer.cancel_brush_stroke();
                    gc.restore();
                }
            }
        });
    }

    ///
    /// Performs a series of painting actions on the canvas
    /// 
    fn paint(&self, _device: &PaintDevice, actions: &Vec<Painting>) {
        // Get the selected layer
        let selected_layer = self.get_selected_layer();

        // ... as a paint layer
        if let Some(selected_layer) = selected_layer {
            let layer_id                                            = selected_layer.id();
            let selected_layer: Option<Editor<PaintLayer+'static>>  = selected_layer.edit();

            // Perform the paint actions on the selected layer if we can
            if let Some(mut selected_layer) = selected_layer {
                for action in actions {
                    self.paint_action(layer_id, &mut *selected_layer, action);
                }

                // If there's a brush stroke waiting, render it
                // Starting a brush stroke selects the layer and creates a save state, which 
                // we assume is still present for the canvas (this is fragile!)
                if selected_layer.has_pending_brush_stroke() {
                    let canvas              = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();
                    let layer: &PaintLayer  = &*selected_layer;

                    canvas.draw(|gc| {
                        // Re-render the current brush stroke
                        gc.restore();
                        gc.store();
                        layer.draw_current_brush_stroke(gc);
                    });
                }
            }
        }
    }
}

impl<Anim: Animation+'static> Controller for CanvasController<Anim> {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.ui_view_model.clone()
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            (PAINT_ACTION, &Paint(ref device, ref painting))    => self.paint(device, painting),
            _                                                   => ()
        };
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<Canvas>>> {
        Some(self.canvases.clone())
    }
}
