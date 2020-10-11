use super::*;
use super::super::tools::*;

use flo_animation::*;

use std::sync::*;

lazy_static! {
    pub static ref SELECTION_TOOLSET_ID: ToolSetId   = ToolSetId(String::from("52E89E39-2955-4330-841C-70E679EA6B5E"));
    pub static ref PAINT_TOOLSET_ID: ToolSetId       = ToolSetId(String::from("4795326D-9A7B-45D0-A6EF-13BD2648699D"));
}

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
    ink:        Arc<FloTool<Anim>>,
    eraser:     Arc<FloTool<Anim>>,
    flood_fill: Arc<FloTool<Anim>>
}

impl<Anim: EditableAnimation+Animation> SelectionTools<Anim> {
    pub fn new() -> SelectionTools<Anim> {
        SelectionTools {
            select: Select::new().to_flo_tool(),
            adjust: Adjust::new().to_flo_tool(),
            pan:    Pan::new().to_flo_tool()
        }
    }
}

impl<Anim: Animation> PaintTools<Anim> {
    pub fn new() -> PaintTools<Anim> {
        PaintTools {
            ink:        Ink::new().to_flo_tool(),
            eraser:     Eraser::new().to_flo_tool(),
            flood_fill: FloodFill::new().to_flo_tool()
        }
    }
}

impl<Anim: Animation> ToolSet<Anim> for SelectionTools<Anim> {
    fn id(&self) -> ToolSetId { SELECTION_TOOLSET_ID.clone() }

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
    fn id(&self) -> ToolSetId { PAINT_TOOLSET_ID.clone() }

    fn set_name(&self) -> String { "Paint".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.ink),
            Arc::clone(&self.eraser),
            Arc::clone(&self.flood_fill)
        ]
    }
}
