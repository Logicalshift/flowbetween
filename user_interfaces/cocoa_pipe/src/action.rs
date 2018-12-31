use super::view_type::*;

///
/// Enumeration of possible actions that can be performed by a Cocoa application
///
pub enum AppAction {
    /// Creates a new window with the specified ID
    CreateWindow(usize),

    /// Ensures that the window with the specified ID is displayed on screen
    OpenWindow(usize),

    /// Sends an action to a window
    Window(usize, WindowAction)
}

///
/// Enumeration of possible actions that can be performed by a Cocoa Window
///
pub enum WindowAction {
    /// Creates a new view of the 
    CreateView(usize, ViewType),

    /// Sets the root view of the window to be the specified view
    SetRootView(usize),

    /// Performs an action on the specified view
    View(usize, ViewAction)
}

///
/// Enumeration of possible actions that can be performed by a Cocoa View
///
pub enum ViewAction {
    /// Removes the view from its superview
    RemoveFromSuperview,

    /// Adds the view with the specified ID as a subview of this view
    AddSubView(usize)
}
