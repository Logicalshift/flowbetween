use super::super::tools::*;

use animation::*;

use std::sync::*;

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
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn create_model(&self) -> () { }

    fn actions_for_input<'a>(&'a self, _data: Option<Arc<()>>, _input: Box<'a+Iterator<Item=ToolInput<()>>>) -> Box<'a+Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
