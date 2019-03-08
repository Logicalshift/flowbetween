use super::action::*;

use flo_ui::*;

///
/// Trait implemented by types that can be converted to a set of app actions
///
pub trait ActionsFrom<TAction> {
    ///
    /// Retrieves the actions required to set up an item of this type in the UI
    ///
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: &mut BindProperty) -> Vec<TAction>;
}
