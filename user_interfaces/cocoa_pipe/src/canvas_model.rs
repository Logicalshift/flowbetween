use flo_ui::*;
use flo_canvas::*;

use std::collections::HashMap;

///
/// Describes the canvases attached to a particular controller
///
pub struct CanvasModel {
    /// The canvas attached to the specified view
    canvas_for_view: HashMap<usize, Resource<BindingCanvas>>,

    /// The views that should receive updates for a particular canvas
    views_with_canvas: HashMap<String, Vec<usize>>
}

impl CanvasModel {
    ///
    /// Creates a new canvas model with no canvases in it
    ///
    pub fn new() -> CanvasModel {
        CanvasModel {
            canvas_for_view: HashMap::new(),
            views_with_canvas: HashMap::new()
        }
    }

    ///
    /// Retrieves the name of a canvas as a string
    ///
    fn name_for_canvas(canvas: &Resource<BindingCanvas>) -> String {
        if let Some(name) = canvas.name() {
            name
        } else {
            format!("{}", canvas.id())
        }
    }

    ///
    /// Associates a canvas with a particular view ID
    ///
    pub fn set_canvas_for_view(&mut self, view_id: usize, canvas: Resource<BindingCanvas>) {
        let canvas_name = Self::name_for_canvas(&canvas);

        self.canvas_for_view.insert(view_id, canvas);
        self.views_with_canvas.entry(canvas_name)
            .or_insert_with(|| vec![])
            .push(view_id);
    }
}
