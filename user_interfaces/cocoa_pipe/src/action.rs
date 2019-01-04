use flo_ui::*;
use flo_canvas::Color;

use super::view_type::*;

///
/// Represents a property binding in a Cocoa application
///
pub enum AppProperty {
    Nothing,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),

    /// Property is bound to a value in the view model
    Bind(usize)
}

///
/// Enumeration of possible actions that can be performed by a Cocoa application
///
#[derive(Clone, PartialEq, Debug)]
pub enum AppAction {
    /// Creates a new window with the specified ID
    CreateWindow(usize),

    /// Sends an action to a window
    Window(usize, WindowAction),

    /// Creates a new view of the specified type
    CreateView(usize, ViewType),

    /// Deletes the view with the specified ID
    DeleteView(usize),

    /// Performs an action on the specified view
    View(usize, ViewAction),

    /// Creates a viewmodel with a particular ID
    CreateViewModel(usize),

    /// Removes the viewmodel with the specified ID
    DeleteViewModel(usize),

    /// Performs an action on the specified view model
    ViewModel(usize, ViewModelAction)
}

///
/// Enumeration of possible actions that can be performed by a Cocoa Window
///
#[derive(Clone, PartialEq, Debug)]
pub enum WindowAction {
    /// Ensures that this window is displayed on screen
    Open,

    /// Sets the root view of the window to be the specified view
    SetRootView(usize),
}

///
/// Enumeration of possible actions that can be performed by a Cocoa View
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewAction {
    /// Removes the view from its superview
    RemoveFromSuperview,

    /// Sets the viewmodel for this view
    SetViewModel(usize),

    /// Adds the view with the specified ID as a subview of this view
    AddSubView(usize),

    /// Sets the bounds of the view for layout
    SetBounds(Bounds),

    /// Sets the Z-Index of the view
    SetZIndex(f64),

    /// Sets the colour of any text or similar element the view might contain
    SetForegroundColor(Color),

    /// Sets the background colour of this view
    SetBackgroundColor(Color)
}

///
/// Enumerationof possible actions for a viewmodel
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewModelAction {
    /// Creates a new viewmodel property with the specified ID
    CreateProperty(usize),

    /// Sets the value of a property to the specified value
    SetPropertyValue(usize, PropertyValue)
}
