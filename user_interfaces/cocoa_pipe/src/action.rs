use flo_ui::*;

use super::view_type::*;

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
    View(usize, ViewAction)
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

    /// Adds the view with the specified ID as a subview of this view
    AddSubView(usize),

    /// Sets the bounds of the view for layout
    SetBounds(Bounds)
}
