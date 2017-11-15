use ui::*;
use ui::canvas::*;
use binding::*;

use std::sync::*;

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController {
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>,
    canvases:   Arc<ResourceManager<Canvas>>
}

impl CanvasController {
    pub fn new() -> CanvasController {
        let canvases = ResourceManager::new();

        let test_canvas = canvases.register(Canvas::new());
        canvases.assign_name(&test_canvas, "test_canvas");

        let ui = bind(Control::canvas().with(test_canvas.clone()));

        test_canvas.draw(|gc| {
            gc.new_path();
            gc.rect(-0.5, -0.5, 0.5, 0.5);
            gc.stroke();
        });

        CanvasController {
            view_model: Arc::new(NullViewModel::new()),
            ui:         ui,
            canvases:   Arc::new(canvases)
        }
    }
}

impl Controller for CanvasController {
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
