use super::action::*;

///
/// Trait implemented by types that can be converted to a set of app actions
///
pub trait ActionsFrom<TAction> {
    ///
    /// Retrieves the actions required to set up an item of this type in the UI
    ///
    fn actions_from(&self) -> Vec<TAction>;
}
