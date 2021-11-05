use crate::image::*;
use crate::control::*;
use crate::property::*;
use crate::binding_canvas::*;
use crate::resource_manager::*;

use flo_binding::*;

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

    /// Adds (or replaces) a named image resource in the resource manager for this controller
    SetImageResource(String, Resource<Image>),

    /// Adds (or replaces) a canvas resource in the resource manager for this controller
    SetCanvasResource(String, Resource<BindingCanvas>),
}
