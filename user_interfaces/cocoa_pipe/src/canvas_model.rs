use std::collections::HashMap;

///
/// Describes the canvases attached to a particular controller
///
pub struct CanvasModel {
    /// The name of the canvas attached to the specified view
    canvas_for_view: HashMap<usize, String>,

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
}