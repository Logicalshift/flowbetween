use super::super::menu::*;
use super::super::tools::*;
use super::super::style::*;
use super::super::viewmodel::*;

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
    ui:                 BindRef<Control>,

    empty_menu:         Arc<EmptyMenuController>,
    ink_menu:           Arc<InkMenuController>
}

impl<Anim: 'static+Animation> MenuController<Anim> {
    ///
    /// Creates a new menu controller
    /// 
    pub fn new(anim_view_model: &AnimationViewModel<Anim>) -> MenuController<Anim> {
        // Create the UI
        let ui          = Self::create_ui(&anim_view_model.tools().effective_tool, &anim_view_model.menu().controller);

        // Create the controllers for the different menu modes
        let brush       = anim_view_model.brush();
        let size        = &brush.size;
        let opacity     = &brush.opacity;
        let color       = &brush.color;

        let empty_menu  = Arc::new(EmptyMenuController::new());
        let ink_menu    = Arc::new(InkMenuController::new(size, opacity, color));

        // Create the controller
        MenuController {
            view_model:         Arc::new(NullViewModel::new()),
            _anim_view_model:   anim_view_model.clone(),
            ui:                 BindRef::from(ui),

            empty_menu:         empty_menu,
            ink_menu:           ink_menu
        }
    }

    ///
    /// Creates the UI binding for this controller
    /// 
    fn create_ui(effective_tool: &BindRef<Option<Arc<Tool<Anim>>>>, tool_controller: &BindRef<String>) -> BindRef<Control> {
        let effective_tool  = effective_tool.clone();
        let tool_controller = tool_controller.clone();

        BindRef::from(computed(move || {
            // Get properties
            let tool_name       = effective_tool.get().map(|tool| tool.tool_name()).unwrap_or("No tool".to_string());
            let tool_controller = tool_controller.get();

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
                        .with(FontAttr::Size(17.0))
                        .with(Bounds::next_horiz(160.0)),
                    
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0))
                        .with(FontAttr::Size(13.0))
                        .with_controller(&tool_controller),
                    
                    Control::label()
                        .with(tool_name)
                        .with(FontAttr::Size(14.0))
                        .with(TextAlign::Center)
                        .with(Bounds::next_horiz(80.0))
                ])
                .with(ControlAttribute::Background(MENU_BACKGROUND))
        }))
    }
}

impl<Anim: Animation> Controller for MenuController<Anim>  {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
        match id {
            INKMENUCONTROLLER   => Some(self.ink_menu.clone()),
            _                   => Some(self.empty_menu.clone())
        }
    }
}
