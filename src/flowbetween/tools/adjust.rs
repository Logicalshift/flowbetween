use super::*;

///
/// The Adjust tool (adjusts control points of existing objects)
/// 
pub struct Adjust { }

impl Adjust {
    ///
    /// Creates a new instance of the Adjust tool
    /// 
    pub fn new() -> Adjust {
        Adjust {}
    }
}

impl Tool for Adjust {
    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn paint(&self, canvas: &Canvas, _selected_layer: Arc<Layer>, _device: &PaintDevice, _actions: &Vec<Painting>) {

    }
}
