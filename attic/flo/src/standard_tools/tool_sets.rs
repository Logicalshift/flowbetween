use super::*;
use crate::tools::*;

use flo_animation::*;

use std::sync::*;

lazy_static! {
    pub static ref CANVAS_TOOLSET_ID: ToolSetId      = ToolSetId(String::from("77F61386-5279-4636-9AA5-D5C1F788E184"));
    pub static ref SELECTION_TOOLSET_ID: ToolSetId   = ToolSetId(String::from("52E89E39-2955-4330-841C-70E679EA6B5E"));
    pub static ref PAINT_TOOLSET_ID: ToolSetId       = ToolSetId(String::from("4795326D-9A7B-45D0-A6EF-13BD2648699D"));
    pub static ref ANIMATION_TOOLSET_ID: ToolSetId   = ToolSetId(String::from("517DB0F4-90BB-4D5C-829D-AAAF46E0119B"));
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
    ink:                Arc<FloTool<Anim>>,
    eraser:             Arc<FloTool<Anim>>,
    shape_ellipse:      Arc<FloTool<Anim>>,
    shape_rectangle:    Arc<FloTool<Anim>>,
    shape_polygon:      Arc<FloTool<Anim>>,
    flood_fill:         Arc<FloTool<Anim>>
}

///
/// The animation toolset
///
pub struct AnimationTools<Anim: 'static+Animation> {
    create_region:      Arc<FloTool<Anim>>
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

impl<Anim: EditableAnimation> PaintTools<Anim> {
    pub fn new() -> PaintTools<Anim> {
        PaintTools {
            ink:                Ink::new().to_flo_tool(),
            eraser:             Eraser::new().to_flo_tool(),
            flood_fill:         FloodFill::new().to_flo_tool(),
            shape_ellipse:      ShapeTool::ellipse().to_flo_tool(),
            shape_rectangle:    ShapeTool::rectangle().to_flo_tool(),
            shape_polygon:      ShapeTool::polygon().to_flo_tool()
        }
    }
}

impl<Anim: EditableAnimation> AnimationTools<Anim> {
    pub fn new() -> AnimationTools<Anim> {
        AnimationTools {
            create_region: CreateAnimationRegion::new().to_flo_tool()
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
            Arc::clone(&self.shape_rectangle),
            Arc::clone(&self.shape_ellipse),
            Arc::clone(&self.shape_polygon),
            Arc::clone(&self.flood_fill)
        ]
    }
}

impl<Anim: Animation> ToolSet<Anim> for AnimationTools<Anim> {
    fn id(&self) -> ToolSetId { ANIMATION_TOOLSET_ID.clone() }

    fn set_name(&self) -> String { "Animation".to_string() }

    fn tools(&self) -> Vec<Arc<FloTool<Anim>>> {
        vec![
            Arc::clone(&self.create_region),
        ]
    }
}
