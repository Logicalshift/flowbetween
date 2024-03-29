use super::edit_bar_controller::*;
use crate::menu::*;
use crate::style::*;
use crate::model::*;
use crate::tools::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::undo::*;

use std::sync::*;
use std::collections::HashMap;

///
/// The menu controller handles the menu at the top of the UI
///
pub struct MenuController<Anim: 'static+EditableAnimation> {
    anim_model:             Arc<FloModel<UndoableAnimation<Anim>>>,
    ui:                     BindRef<Control>,
    tool_controllers:       Mutex<HashMap<String, Arc<dyn Controller>>>,
    edit_bar_controller:    Arc<dyn Controller>,

    empty_menu:             Arc<EmptyMenuController>
}

impl<Anim: 'static+EditableAnimation> MenuController<Anim> {
    ///
    /// Creates a new menu controller
    ///
    pub fn new(anim_model: &FloModel<UndoableAnimation<Anim>>) -> MenuController<Anim> {
        // Create the UI
        let effective_tool          = anim_model.tools().effective_tool.clone();
        let tool_controller         = BindRef::from(computed(move || format!("Tool_{}", effective_tool.get().map(|tool| tool.tool_name()).unwrap_or(String::new()))));
        let ui                      = Self::create_ui(&tool_controller);
        let empty_menu              = Arc::new(EmptyMenuController::new());
        let anim_model              = Arc::new(anim_model.clone());
        let edit_bar_controller     = Arc::new(edit_bar_controller(&anim_model));

        // Create the controller
        MenuController {
            anim_model:             anim_model,
            ui:                     BindRef::from(ui),
            tool_controllers:       Mutex::new(HashMap::new()),
            edit_bar_controller:    edit_bar_controller,

            empty_menu:             empty_menu,
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
                    // LHS controls
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

                    // Tool controls
                    Control::empty()
                        .with(Bounds::stretch_horiz(1.0))
                        .with(Font::Size(12.0))
                        .with_controller(&tool_controller),

                    // RHS controls
                    Control::empty()
                        .with(Bounds::next_horiz(240.0))
                        .with_controller("EditBarController")
                ])
                .with(Appearance::Background(MENU_BACKGROUND))
        }))
    }

    ///
    /// Given a controller name (something like Tool_Foo), finds the tool that manages it
    ///
    fn tool_for_controller_name(&self, controller_name: &str) -> Option<Arc<FloTool<UndoableAnimation<Anim>>>> {
        // Go through the tool sets and find the first that matches the name
        let tool_sets = self.anim_model.tools().tool_sets.get();
        tool_sets.into_iter()
            .flat_map(|set| set.tools().into_iter())
            .filter(|tool|  format!("Tool_{}", tool.tool_name()) == controller_name)
            .nth(0)
    }
}

impl<Anim: 'static+EditableAnimation> Controller for MenuController<Anim>  {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        use std::collections::hash_map::Entry::*;

        // Internal controllers
        if id == "EditBarController" {
            return Some(Arc::clone(&self.edit_bar_controller));
        }

        // Try to fetch the existing controller for this ID
        let mut tool_controllers    = self.tool_controllers.lock().unwrap();
        let entry                   = tool_controllers.entry(id.to_string());

        match entry {
            // Occupied entries just map to the controller
            Occupied(controller) => Some(Arc::clone(controller.get())),

            // Vacant entries create a new controller if possible (caching it away so it becomes permanent)
            Vacant(no_controller) => {
                // Try to find the tool with this controller
                let tool = self.tool_for_controller_name(id);

                if let Some(tool) = tool {
                    // Tool exists: create the controller
                    let tool_model  = self.anim_model.tools().model_for_tool(&*tool, Arc::clone(&self.anim_model));
                    let controller  = tool.create_menu_controller(Arc::clone(&self.anim_model), &*tool_model);

                    if let Some(controller) = controller {
                        // Store in the entry
                        no_controller.insert(Arc::clone(&controller));

                        // Make this the result
                        Some(controller)
                    } else {
                        // No controller for this tool
                        Some(self.empty_menu.clone())
                    }
                } else {
                    // Unknown tool
                    None
                }
            }
        }
    }
}
