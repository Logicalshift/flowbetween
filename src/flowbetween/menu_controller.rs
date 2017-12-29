use super::style::*;
use super::viewmodel::*;

use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// The menu controller handles the menbu at the top of the UI
///
pub struct MenuController<Anim: Animation> {
    view_model:         Arc<NullViewModel>,
    _anim_view_model:   AnimationViewModel<Anim>,
    ui:                 Arc<Bound<Control>>
}

impl<Anim: 'static+Animation> MenuController<Anim> {
    ///
    /// Creates a new menu controller
    /// 
    pub fn new(anim_view_model: &AnimationViewModel<Anim>) -> MenuController<Anim> {
        // Generate the UI for the menu
        let selected_tool = anim_view_model.tools().effective_tool.clone();

        let ui = computed(move || {
            // Get properties
            let tool_name = selected_tool.get().map(|tool| tool.tool_name()).unwrap_or("No tool".to_string());

            // The control tree for the menu
            Control::empty()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::empty()
                        .with(Bounds::next_horiz(6.0))
                        .with(ControlAttribute::Background(MENU_BACKGROUND_ALT)),
                    Control::empty()
                        .with(Bounds::next_horiz(2.0)),
                    Control::label()
                        .with("FlowBetween")
                        .with(ControlAttribute::FontSize(17.0))
                        .with(Bounds::next_horiz(160.0)),
                    
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0)),
                    
                    Control::label()
                        .with(tool_name)
                        .with(ControlAttribute::FontSize(14.0))
                        .with(Bounds::next_horiz(80.0))
                ])
                .with(ControlAttribute::Background(MENU_BACKGROUND))
        });

        // Create the controller
        MenuController {
            view_model:         Arc::new(NullViewModel::new()),
            _anim_view_model:   anim_view_model.clone(),
            ui:                 Arc::new(ui)
        }
    }
}

impl<Anim: Animation>  Controller for MenuController<Anim>  {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::clone(&self.ui)
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
