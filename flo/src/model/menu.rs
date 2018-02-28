use super::super::tools::*;

use binding::*;
use animation::*;

use std::sync::*;

///
/// The viewmodel for the menu bar
/// 
#[derive(Clone)]
pub struct MenuModel {
    /// The controller to use for the menu bar
    pub controller: BindRef<String>
}

impl MenuModel {
    ///
    /// Creates a new menu view model
    /// 
    pub fn new<Anim: 'static+Animation>(effective_tool: &BindRef<Option<Arc<FloTool<Anim>>>>) -> MenuModel {
        let controller = Self::controller_for_tool(effective_tool.clone());

        MenuModel {
            controller: controller
        }
    }

    ///
    /// Creates a binding that returns the menu controller to use when a particular tool is selected
    /// 
    fn controller_for_tool<Anim: 'static+Animation>(tool: BindRef<Option<Arc<FloTool<Anim>>>>) -> BindRef<String> {
        BindRef::from(computed(move || {
            tool.get().map(|tool| tool.menu_controller_name()).unwrap_or("".to_string())
        }))
    }
}