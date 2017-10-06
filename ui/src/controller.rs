use super::binding::*;
use super::control::*;

///
/// Controllers represent a portion of the UI and provide a hub for
/// receiving events related to it and connecting the model or
/// viewmodel.
///
/// The subcontroller identifier can be used to retrieve controllers
/// that deal with a different part of the UI.
///
pub trait Controller<SubControllerIdentifier> {
    /// Retrieves a Control representing the UI for this controller
    fn ui() -> Box<Bound<Control>>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(id: SubControllerIdentifier) /* -> ??? - needs to be Option<Box<Controller>> but we want an untyped version here */;
}
