use super::super::tools::*;

use ui::*;
use animation::*;

use std::sync::*;

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

impl<Anim: Animation> Tool<Anim> for Pencil {
    fn tool_name(&self) -> String { "Pencil".to_string() }

    fn image_name(&self) -> String { "pencil".to_string() }

    fn paint<'a>(&self, _model: &ToolModel<'a, Anim>, _device: &PaintDevice, _actions: &Vec<Painting>) {
        
    }
}

impl<Anim: Animation> Tool2<(), Anim> for Pencil {
    fn tool_name(&self) -> String { "Pencil".to_string() }

    fn image_name(&self) -> String { "pencil".to_string() }

    fn actions_for_input<'a>(&'a self, _data: Option<Arc<()>>, _input: Box<'a+Iterator<Item=ToolInput<()>>>) -> Box<Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
