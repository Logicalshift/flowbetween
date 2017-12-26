use super::*;

///
/// The Pan tool (Pans control points of existing objects)
/// 
pub struct Pan { }

impl Pan {
    ///
    /// Creates a new instance of the Pan tool
    /// 
    pub fn new() -> Pan {
        Pan {}
    }
}

impl Tool for Pan {
    fn tool_name(&self) -> String { "Pan".to_string() }

    fn image_name(&self) -> String { "pan".to_string() }

    fn paint(&self, canvas: &Canvas, _selected_layer: Arc<Layer>, _device: &PaintDevice, _actions: &Vec<Painting>) {
        
    }
}
