use super::*;
use super::super::tools::*;

use animation::*;

use std::sync::*;

///
/// The selection toolset
/// 
pub struct SelectionTools<Anim: 'static+Animation> {
    select: Arc<FloTool<Anim>>,
    adjust: Arc<FloTool<Anim>>,
    pan:    Arc<FloTool<Anim>>
}

///
/// The paint toolset
/// 
pub struct PaintTools<Anim: 'static+Animation> {
    pencil: Arc<FloTool<Anim>>,
    ink:    Arc<FloTool<Anim>>,
    eraser: Arc<FloTool<Anim>>
}

impl<Anim: Animation> SelectionTools<Anim> {
    pub fn new() -> SelectionTools<Anim> {
        SelectionTools {
            select: Select::new().to_generic_tool(),
            adjust: Adjust::new().to_generic_tool(),
            pan:    Pan::new().to_generic_tool()
        }
    }
}

impl<Anim: Animation> PaintTools<Anim> {
    pub fn new() -> PaintTools<Anim> {
        PaintTools {
            pencil: Pencil::new().to_generic_tool(),
            ink:    Ink::new().to_generic_tool(),
            eraser: Eraser::new().to_generic_tool()
        }
    }
}

impl<Anim: Animation> ToolSet<Anim> for SelectionTools<Anim> {
    fn set_name(&self) -> String { "Selection".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.select),
            Arc::clone(&self.adjust),
            Arc::clone(&self.pan)
        ]
    }
}

impl<Anim: Animation> ToolSet<Anim> for PaintTools<Anim> {
    fn set_name(&self) -> String { "Paint".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.pencil),
            Arc::clone(&self.ink),
            Arc::clone(&self.eraser)
        ]
    }
}
