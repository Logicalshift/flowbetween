use super::binding::*;
use super::control::*;

// TODO: it would be nice to use specific enum types for
// sub-controllers and events. However, this causes a few
// problems. Firstly, it's pretty tricky to add them as
// parameters for Control due to the need to be clonable,
// serializable, etc. Secondly, it would probably mean that
// all controllers and sub-controllers would need the same
// type (I suppose there could be some kind of controller
// that takes two controllers and makes them the same
// type but it's still a lot of work just for a slightly
// nicer system)
//
// Can't use Any as it's not really compatible with cloning 
// and serialization
//
// For this reason, events and subcontrollers are identified
// by strings which need to be parsed.

///
/// Controllers represent a portion of the UI and provide a hub for
/// receiving events related to it and connecting the model or
/// viewmodel.
///
pub trait Controller {
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> Box<Bound<Control>>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, id: &str) -> Option<Box<Controller>>;
}
