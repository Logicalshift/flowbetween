use gtk::*;

/// ID used to identify a Gtk window
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum WindowId {
    Unassigned,
    Assigned(i64)
}

/// ID used to identify a Gtk widget
pub enum WidgetId {
    Unassigned,
    Assigned(i64)
}

///
/// Actions that cacn be performed on a window
/// 
pub enum GtkWindowAction {
    New(WindowType),
    SetPosition(WindowPosition),
    SetDefaultSize(i32, u32),
    SetTitle(String),
    ShowAll,
    Hide,
    Close
}

///
/// GTK actions that can be requested
/// 
pub enum GtkAction {
    /// Shuts down Gtk
    Stop,

    /// Performs an action on a window
    Window(WindowId, GtkWindowAction)
}
