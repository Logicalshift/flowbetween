use super::*;
use super::super::tools::*;

use flo_animation::*;

use std::sync::*;

lazy_static! {
    pub static ref CANVAS_TOOLSET_ID: ToolSetId      = ToolSetId(String::from("77F61386-5279-4636-9AA5-D5C1F788E184"));
    pub static ref SELECTION_TOOLSET_ID: ToolSetId   = ToolSetId(String::from("52E89E39-2955-4330-841C-70E679EA6B5E"));
    pub static ref PAINT_TOOLSET_ID: ToolSetId       = ToolSetId(String::from("4795326D-9A7B-45D0-A6EF-13BD2648699D"));
}

pub struct CanvasTools<Anim: 'static+Animation> {
    pan:    Arc<FloTool<Anim>>
}

///
/// The selection toolset
///
pub struct SelectionTools<Anim: 'static+Animation> {
    select: Arc<FloTool<Anim>>,
    adjust: Arc<FloTool<Anim>>,
    lasso: Arc<FloTool<Anim>>
}

///
/// The paint toolset
///
pub struct PaintTools<Anim: 'static+Animation> {
    ink:        Arc<FloTool<Anim>>,
    eraser:     Arc<FloTool<Anim>>,
    flood_fill: Arc<FloTool<Anim>>
}

impl<Anim: EditableAnimation+Animation> CanvasTools<Anim> {
    pub fn new() -> CanvasTools<Anim> {
        CanvasTools {
            pan:    Pan::new().to_flo_tool()
        }
    }
}

impl<Anim: EditableAnimation+Animation> SelectionTools<Anim> {
    pub fn new() -> SelectionTools<Anim> {
        SelectionTools {
            select: Select::new().to_flo_tool(),
            adjust: Adjust::new().to_flo_tool(),
            lasso: Lasso::new().to_flo_tool()
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

impl<Anim: Animation> ToolSet<Anim> for CanvasTools<Anim> {
    fn id(&self) -> ToolSetId { CANVAS_TOOLSET_ID.clone() }

    fn set_name(&self) -> String { "Canvas".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.pan)
        ]
    }
}

impl<Anim: Animation> ToolSet<Anim> for SelectionTools<Anim> {
    fn id(&self) -> ToolSetId { SELECTION_TOOLSET_ID.clone() }

    fn set_name(&self) -> String { "Selection".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.select),
            Arc::clone(&self.adjust),
            Arc::clone(&self.lasso)
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
