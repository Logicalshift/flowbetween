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
                    x1: After,
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
                    x1: After,
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
                    x1: After,
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
        Box::new(computed(|| {
            Control::container()
        }))
    }

    ///
    /// Creates the UI tree for this session
    ///
    pub fn ui_tree(&self) -> Box<Bound<Control>> {
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
                        .with(vec![toolbar.get(), canvas.get()]),
                    timeline.get()])
        }))
    }
}

impl Session for FlowBetweenSession {
    /// Creates a new session
    fn start_new(state: Arc<SessionState>) -> Self {
        // Create the UI
        let session = FlowBetweenSession::new();

        state.set_ui_tree(session.ui_tree());

        session
    }
}