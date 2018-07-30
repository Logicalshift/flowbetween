use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Controller that provides controls for adding/deleting/editing layers (generally displayed above the main layer list)
/// 
pub struct TimelineLayerControlsController {
    /// the UI for this controller
    ui: BindRef<Control>
}

impl TimelineLayerControlsController {
    ///
    /// Creates a new timeline layer controls controller
    /// 
    pub fn new<Anim: 'static+Animation>(_model: &FloModel<Anim>) -> TimelineLayerControlsController {
        let ui = Self::ui();

        TimelineLayerControlsController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for the layer controls controller
    /// 
    fn ui() -> BindRef<Control> {
        // Create the UI
        let ui = computed(move || {
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::container()
                        .with(Font::Size(13.0))
                        .with(Font::Weight(FontWeight::ExtraBold))
                        .with(ControlAttribute::Padding((4, 1), (4, 1)))
                        .with(vec![
                            Control::empty()
                                .with(Bounds::stretch_horiz(1.0)),
                            Control::label()
                                .with(Bounds::next_horiz(14.0))
                                .with(TextAlign::Center)
                                .with((ActionTrigger::Click, "AddNewLayer"))
                                .with("+"),
                            Control::label()
                                .with(Bounds::next_horiz(14.0))
                                .with(TextAlign::Center)
                                .with((ActionTrigger::Click, "RemoveLayer"))
                                .with("-"),
                        ])
                        .with(Bounds::stretch_vert(1.0)),
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_BORDER))
                        .with(Bounds::next_vert(1.0))
                ])
        });

        // Turn into a bindref
        BindRef::from(ui)
    }
}

impl Controller for TimelineLayerControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }
}