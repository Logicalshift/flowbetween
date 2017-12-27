use super::*;

///
/// The selection toolset
/// 
pub struct SelectionTools<Anim: 'static+Animation> {
    select: Arc<Tool<Anim>>,
    adjust: Arc<Tool<Anim>>,
    pan:    Arc<Tool<Anim>>
}

///
/// The paint toolset
/// 
pub struct PaintTools<Anim: 'static+Animation> {
    pencil: Arc<Tool<Anim>>,
    ink:    Arc<Tool<Anim>>
}

impl<Anim: Animation> SelectionTools<Anim> {
    pub fn new() -> SelectionTools<Anim> {
        SelectionTools {
            select: Arc::new(Select::new()),
            adjust: Arc::new(Adjust::new()),
            pan:    Arc::new(Pan::new())
        }
    }
}

impl<Anim: Animation> PaintTools<Anim> {
    pub fn new() -> PaintTools<Anim> {
        PaintTools {
            pencil: Arc::new(Pencil::new()),
            ink:    Arc::new(Ink::new())
        }
    }
}

impl<Anim: Animation> ToolSet<Anim> for SelectionTools<Anim> {
    fn set_name(&self) -> String { "Selection".to_string() }

    fn tools(&self) -> Vec<Arc<Tool<Anim>>> {
        vec![
            Arc::clone(&self.select),
            Arc::clone(&self.adjust),
            Arc::clone(&self.pan)
        ]
    }
}

impl<Anim: Animation> ToolSet<Anim> for PaintTools<Anim> {
    fn set_name(&self) -> String { "Paint".to_string() }

    fn tools(&self) -> Vec<Arc<Tool<Anim>>> {
        vec![
            Arc::clone(&self.pencil),
            Arc::clone(&self.ink),
        ]
    }
}
