use ui::*;
use http_ui::*;

use std::sync::*;

///
/// The main flowbetween session object
///
pub struct FlowBetweenSession {

}

impl FlowBetweenSession {
    pub fn new() -> FlowBetweenSession {
        FlowBetweenSession {
        }
    }

    ///
    /// Creates the menu bar control for this session
    ///
    pub fn menu_bar(&self) -> Box<Bound<Control>> {
        use ui::Position::*;

        Box::new(computed(|| {
            Control::container()
                .with(Bounds {
                    x1: Start,
                    y1: After,
                    x2: End,
                    y2: Offset(32.0)
                })
        }))
    }

    ///
    /// Creates the timeline control
    ///
    pub fn timeline(&self) -> Box<Bound<Control>> {
        use ui::Position::*;

        Box::new(computed(|| {
            Control::container()
                .with(Bounds {
                    x1: Start,
                    y1: After,
                    x2: End,
                    y2: Offset(256.0)
                })
        }))
    }

    ///
    /// Creates the toolbar control
    ///
    pub fn toolbar(&self) -> Box<Bound<Control>> {
        use ui::Position::*;

        Box::new(computed(|| {
            Control::container()
                .with(Bounds {
                    x1: Start,
                    y1: After,
                    x2: Offset(48.0),
                    y2: End                    
                })
        }))
    }

    ///
    /// Creates the canvas control
    ///
    pub fn canvas(&self) -> Box<Bound<Control>> {
        use ui::Position::*;

        Box::new(computed(|| {
            Control::container()
                .with(Bounds {
                    x1: After,
                    y1: Start,
                    x2: Stretch(1.0),
                    y2: End
                })
        }))
    }

    ///
    /// Creates the UI tree for this session
    ///
    pub fn ui_tree(&self) -> Box<Bound<Control>> {
        use ui::Position::*;

        let menu_bar    = self.menu_bar();
        let timeline    = self.timeline();
        let toolbar     = self.toolbar();
        let canvas      = self.canvas();

        Box::new(computed(move || {
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    menu_bar.get(),
                    Control::container()
                        .with((vec![toolbar.get(), canvas.get()],
                            Bounds { x1: Start, y1: After, x2: End, y2: Stretch(1.0) })),
                    timeline.get()])
        }))
    }
}

impl Session for FlowBetweenSession {
    /// Creates a new session
    fn start_new(_state: Arc<SessionState>) -> Self {
        let session = FlowBetweenSession::new();

        session
    }
}

impl Controller for FlowBetweenSession {
    fn ui(&self) -> Box<Bound<Control>> {
        self.ui_tree()
    }

    fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
        None
    }
}
