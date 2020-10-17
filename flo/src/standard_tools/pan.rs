use super::super::tools::*;
use super::super::model::*;

use flo_ui::*;
use flo_animation::*;

use std::sync::*;

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

impl<Anim: Animation> Tool<Anim> for Pan {
    type ToolData   = ();
    type Model      = ();

    fn tool_name(&self) -> String { "Pan".to_string() }

    fn image(&self) -> Option<Image> { Some(svg_static(include_bytes!("../../svg/tools/pan.svg"))) }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<()>>, _input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn Iterator<Item=ToolAction<()>>> {
        Box::new(vec![].into_iter())
    }
}
