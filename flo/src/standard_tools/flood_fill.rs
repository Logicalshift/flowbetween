use super::super::tools::*;
use super::super::model::*;

use flo_animation::*;

use std::sync::*;

///
/// A tool for flood-filling areas of the canvas
///
pub struct FloodFill {

}

impl FloodFill {
    ///
    /// Creates a new flood-fill tool
    ///
    pub fn new() -> FloodFill {
        FloodFill {
        }
    }
}

impl<Anim: Animation> Tool<Anim> for FloodFill {
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Flood Fill".to_string() }

    fn image_name(&self) -> String { "floodfill".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<()>>, _input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
