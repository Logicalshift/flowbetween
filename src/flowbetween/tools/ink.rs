use super::*;

///
/// The Ink tool (Inks control points of existing objects)
/// 
pub struct Ink { }

impl Ink {
    ///
    /// Creates a new instance of the Ink tool
    /// 
    pub fn new() -> Ink {
        Ink {}
    }
}

impl Tool for Ink {
    fn tool_name(&self) -> String { "Ink".to_string() }

    fn image_name(&self) -> String { "ink".to_string() }

    fn paint(&self, _selected_layer: Arc<Layer>, _device: &PaintDevice, _actions: &Vec<Painting>) {

    }
}
