use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;

use ui::*;
use animation::*;

use std::sync::*;

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
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Select".to_string() }

    fn image_name(&self) -> String { "select".to_string() }

    fn create_model(&self) -> () { }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Option<Arc<Controller>> {
        Some(Arc::new(SelectMenuController::new()))
    }

    fn actions_for_input<'a>(&self, _data: Option<Arc<()>>, _input: Box<'a+Iterator<Item=ToolInput<()>>>) -> Box<Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
