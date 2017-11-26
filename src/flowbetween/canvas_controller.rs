use ui::*;
use ui::canvas::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController<Anim: EditableAnimation> {
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>,
    canvases:   Arc<ResourceManager<Canvas>>,
    animation:  Arc<Anim>
}

impl<Anim: EditableAnimation> CanvasController<Anim> {
    pub fn new(animation: &Arc<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases = ResourceManager::new();

        // Create the controller
        let mut controller = CanvasController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         bind(Control::empty()),
            canvases:   Arc::new(canvases),
            animation:  animation.clone()
        };

        // Set up the UI
        let background_canvas = controller.create_background_canvas();

        controller.ui.set(Control::canvas()
            .with(background_canvas)
            .with(Bounds::fill_all())
            .with((
                (ActionTrigger::Paint(PaintDevice::Pen),                        "Paint"),
                (ActionTrigger::Paint(PaintDevice::Touch),                      "Paint"),
                (ActionTrigger::Paint(PaintDevice::Other),                      "Paint"),
                (ActionTrigger::Paint(PaintDevice::Mouse(MouseButton::Left)),   "Paint")
            )));

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
        let (width, height) = self.animation.size();

        canvas.draw(move |gc| {
            gc.clear_canvas();
            gc.canvas_height((height*1.05) as f32);
            gc.center_region(0.0,0.0, width as f32, height as f32);
        });
    }

    ///
    /// Create the background canvas for this controller
    ///
    fn create_background_canvas(&self) -> Resource<Canvas> {
        let canvas          = self.create_canvas();
        let (width, height) = self.animation.size();

        canvas.draw(move |gc| {
            let (width, height)             = (width as f32, height as f32);

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
        });

        canvas
    }
}

impl<Anim: EditableAnimation> Controller for CanvasController<Anim> {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        println!("{:?} {:?}", action_id, action_parameter);
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<Canvas>>> {
        Some(self.canvases.clone())
    }
}
