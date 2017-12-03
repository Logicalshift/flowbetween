use super::viewmodel::*;

use ui::*;
use ui::canvas::*;
use binding::*;
use animation::*;

use std::sync::*;
use rand;
use rand::Rng;

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController<Anim: Animation> {
    ui_view_model:      Arc<NullViewModel>,
    ui:                 Binding<Control>,
    canvases:           Arc<ResourceManager<Canvas>>,
    anim_view_model:    AnimationViewModel<Anim>
}

impl<Anim: Animation> CanvasController<Anim> {
    pub fn new(view_model: &AnimationViewModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases = ResourceManager::new();

        // Create the controller
        let mut controller = CanvasController {
            ui_view_model:      Arc::new(NullViewModel::new()),
            ui:                 bind(Control::empty()),
            canvases:           Arc::new(canvases),
            anim_view_model:    view_model.clone()
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
    /// Create the background canvas for this controller
    ///
    fn create_background_canvas(&self) -> Resource<Canvas> {
        let canvas          = self.create_canvas();
        let (width, height) = open_read::<AnimationSize>(self.anim_view_model.animation())
            .map(|size| size.size())
            .unwrap_or((1920.0, 1080.0));

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

            // Draw some random crap
            gc.store();

            let mut rng = rand::thread_rng();
            for _ in 0..100 {
                gc.new_path();
                gc.fill_color(Color::Rgba(rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), 1.0));

                gc.rect(rng.gen_range(0.0, 1920.0), rng.gen_range(0.0, 1080.0), rng.gen_range(0.0, 1920.0), rng.gen_range(0.0, 1080.0));

                gc.fill();

                if rng.gen() {
                    gc.restore();
                    gc.store();
                }
            }
        });

        canvas
    }
}

impl<Anim: Animation> Controller for CanvasController<Anim> {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.ui_view_model.clone()
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        println!("{:?} {:?}", action_id, action_parameter);
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<Canvas>>> {
        Some(self.canvases.clone())
    }
}
