use super::super::menu::*;
use super::super::style::*;
use super::super::model::*;

use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// The menu controller handles the menbu at the top of the UI
///
pub struct MenuController<Anim: Animation> {
    _anim_view_model:   FloModel<Anim>,
    ui:                 BindRef<Control>,

    empty_menu:         Arc<EmptyMenuController>,
    ink_menu:           Arc<InkMenuController>
}

impl<Anim: 'static+Animation> MenuController<Anim> {
    ///
    /// Creates a new menu controller
    /// 
    pub fn new(anim_view_model: &FloModel<Anim>) -> MenuController<Anim> {
        // Create the UI
        let ui          = Self::create_ui(&anim_view_model.menu().controller);

        // Create the controllers for the different menu modes
        let brush       = anim_view_model.brush();
        let size        = &brush.size;
        let opacity     = &brush.opacity;
        let color       = &brush.color;

        let empty_menu  = Arc::new(EmptyMenuController::new());
        let ink_menu    = Arc::new(InkMenuController::new(size, opacity, color));

        // Create the controller
        MenuController {
            _anim_view_model:   anim_view_model.clone(),
            ui:                 BindRef::from(ui),

            empty_menu:         empty_menu,
            ink_menu:           ink_menu
        }
    }

    ///
    /// Creates the UI binding for this controller
    /// 
    fn create_ui(tool_controller: &BindRef<String>) -> BindRef<Control> {
        let tool_controller = tool_controller.clone();

        BindRef::from(computed(move || {
            // Get properties
            let tool_controller = tool_controller.get();

            // The control tree for the menu
            Control::empty()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::empty()
                        .with(Bounds::next_horiz(6.0))
                        .with(Appearance::Background(MENU_BACKGROUND_ALT)),
                    Control::empty()
                        .with(Bounds::next_horiz(2.0)),
                    Control::label()
                        .with("FlowBetween")
                        .with(FontWeight::Light)
                        .with(Font::Size(17.0))
                        .with(Bounds::next_horiz(160.0)),
                    
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0))
                        .with(Font::Size(12.0))
                        .with_controller(&tool_controller),
                ])
                .with(Appearance::Background(MENU_BACKGROUND))
        }))
    }
}

impl<Anim: Animation> Controller for MenuController<Anim>  {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
        match id {
            INKMENUCONTROLLER   => Some(self.ink_menu.clone()),
            _                   => Some(self.empty_menu.clone())
        }
    }
}
