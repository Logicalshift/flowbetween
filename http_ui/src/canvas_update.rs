///
/// Data stored for a canvas update event
///
#[derive(Serialize, Deserialize, Clone)]
pub struct CanvasUpdate {
    ///
    /// The path to the controller for this canvas
    ///
    controller_path: Vec<String>,

    ///
    /// The canvas that is being updated
    ///
    canvas_id: String,

    ///
    /// The updates that should be applied for this canvas 
    ///
    updates: String
}

impl CanvasUpdate {
    pub fn new(controller_path: Vec<String>, canvas_id: String, updates: String) -> CanvasUpdate {
        CanvasUpdate {
            controller_path:    controller_path,
            canvas_id:          canvas_id,
            updates:            updates
        }
    }
}