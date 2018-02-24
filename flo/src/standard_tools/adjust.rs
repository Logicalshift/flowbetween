use super::super::tools::*;

use ui::*;
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

    fn paint<'a>(&self, _model: &ToolModel<'a, Anim>, _device: &PaintDevice, _actions: &Vec<Painting>) {

    }
}

impl<Anim: Animation> Tool2<(), Anim> for Adjust {
    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn actions_for_input<'b>(&self, _data: Option<&'b ()>, _input: Box<Iterator<Item=ToolInput<'b, ()>>>) -> Box<Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
