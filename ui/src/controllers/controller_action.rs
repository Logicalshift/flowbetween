use crate::image::*;
use crate::control::*;
use crate::property::*;
use crate::controller::*;
use crate::binding_canvas::*;
use crate::resource_manager::*;

use flo_binding::*;

use std::sync::*;

///
/// The possible actions that a StreamController can perform
///
pub enum ControllerAction {
    /// Sets the UI binding for the controller
    SetUi(BindRef<Control>),

    /// Sets the value of a property in the viewmodel for this controller
    SetProperty(String, PropertyValue),

    /// Sets the value of a property to a bound value in this controller
    SetPropertyBinding(String, BindRef<PropertyValue>),

    /// Adds a subcontroller with the specified name to the controller
    AddSubController(String, Arc<dyn Controller>),

    /// Removes a subcontroller from this controller
    RemoveSubController(String),
}
