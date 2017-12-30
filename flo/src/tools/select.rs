use super::*;

///
/// The Select tool (Selects control points of existing objects)
/// 
pub struct Select { }

impl Select {
    ///
    /// Creates a new instance of the Select tool
    /// 
    pub fn new() -> Select {
        Select {}
    }
}

impl<Anim: Animation> Tool<Anim> for Select {
    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    fn paint<'a>(&self, _model: &ToolModel<'a, Anim>, _device: &PaintDevice, _actions: &Vec<Painting>) {
        
    }
}
