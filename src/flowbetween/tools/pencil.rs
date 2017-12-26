use super::*;

///
/// The Pencil tool (Pencils control points of existing objects)
/// 
pub struct Pencil { }

impl Pencil {
    ///
    /// Creates a new instance of the Pencil tool
    /// 
    pub fn new() -> Pencil {
        Pencil {}
    }
}

impl Tool for Pencil {
    fn tool_name(&self) -> String { "Pencil".to_string() }

    fn image_name(&self) -> String { "pencil".to_string() }

    fn paint(&self, canvas: &Canvas, _selected_layer: Arc<Layer>, _device: &PaintDevice, _actions: &Vec<Painting>) {
        
    }
}
