///
/// Data stored for a canvas update event
///
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CanvasUpdate {
    ///
    /// The path to the controller for this canvas (using the same encoding we use for flo-canvas)
    ///
    controller: String,

    ///
    /// The canvas that is being updated
    ///
    canvas_name: String,

    ///
    /// The updates that should be applied for this canvas
    ///
    updates: String
}

impl CanvasUpdate {
    pub fn new(controller: String, canvas_name: String, updates: String) -> CanvasUpdate {
        CanvasUpdate {
            controller:     controller,
            canvas_name:    canvas_name,
            updates:        updates
        }
    }
}
