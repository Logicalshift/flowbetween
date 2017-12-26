use super::*;
use animation::*;

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

impl<Anim: Animation> Tool<Anim> for Adjust {
    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn paint<'a>(&self, model: &ToolModel<'a, Anim>, device: &PaintDevice, actions: &Vec<Painting>) {

    }
}
