use super::super::tools::*;
use super::super::model::*;

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
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Pencil".to_string() }

    fn image_name(&self) -> String { "pencil".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<()>>, _input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
