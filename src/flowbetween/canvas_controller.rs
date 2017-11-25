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
        let canvases = ResourceManager::new();

        let test_canvas = canvases.register(Canvas::new());
        canvases.assign_name(&test_canvas, "test_canvas");

        let ui = bind(Control::canvas()
            .with(test_canvas.clone())
            .with(Bounds::fill_all()));

        test_canvas.draw(|gc| {
            gc.line_width_pixels(2.0);
            gc.fill_color(Color::Rgba(0.5,0.0,0.0, 1.0));
            gc.stroke_color(Color::Rgba(0.0,0.0,0.0, 1.0));

            gc.new_path();
            gc.rect(-0.5, -0.5, 0.5, 0.5);
            gc.fill();
            gc.stroke();
        });

        CanvasController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         ui,
            canvases:   Arc::new(canvases),
            animation:  animation.clone()
        }
    }
}

impl<Anim: EditableAnimation> Controller for CanvasController<Anim> {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<Canvas>>> {
        Some(self.canvases.clone())
    }
}
