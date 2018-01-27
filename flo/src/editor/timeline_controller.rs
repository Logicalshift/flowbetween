use super::super::style::*;

use ui::*;
use canvas::*;
use binding::*;

use std::sync::*;

///
/// The timeline allows the user to pick a point in time and create layers in the animation
///
pub struct TimelineController {
    /// The canvases for the timeline
    canvases:           Arc<ResourceManager<BindingCanvas>>,

    /// The UI for the timeline
    ui:                 Binding<Control>
}

impl TimelineController {
    ///
    /// Creates a new timeline controller
    /// 
    pub fn new() -> TimelineController {
        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        // UI
        let ui = bind(Control::scrolling_container()
            .with(Bounds::fill_all())
            .with(Scroll::MinimumContentSize(6000.0, 16.0))
            .with(Scroll::HorizontalScrollBar(ScrollBarVisibility::Always))
            .with(Scroll::VerticalScrollBar(ScrollBarVisibility::OnlyIfNeeded))
            .with(Appearance::Background(TIMELINE_BACKGROUND))
            .with(ControlAttribute::Padding((16, 16), (16, 16)))
            .with(vec![
                Control::empty()
                    .with(Appearance::Background(Color::Rgba(0.4, 0.0, 0.0, 1.0)))
                    .with(Bounds::fill_all())
            ])
            .with((ActionTrigger::VirtualScroll(400.0, 256.0), "Scroll")));

        // Piece it together
        TimelineController {
            ui:         ui,
            canvases:   canvases
        }
    }
}

impl Controller for TimelineController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(Arc::clone(&self.canvases))
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            ("Scroll", &VirtualScroll((x, y), (width, height))) => {
                println!("{:?} {:?}", (x, y), (width, height));
            },

            _ => ()
        }
    }
}
