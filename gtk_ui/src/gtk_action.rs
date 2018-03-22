use gtk::*;

/// ID used to identify a Gtk window
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum WindowId {
    Unassigned,
    Assigned(i64)
}

/// ID used to identify a Gtk widget
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum WidgetId {
    Unassigned,
    Assigned(i64)
}

///
/// Actions that can be performed on a window
/// 
#[derive(Clone)]
pub enum GtkWindowAction {
    New(WindowType),
    SetPosition(WindowPosition),
    SetDefaultSize(i32, i32),
    SetTitle(String),
    ShowAll,
    Hide,
    Close
}

///
/// Types of widget that can be created
/// 
#[derive(Clone)]
pub enum GtkWidgetType {
    Layout,
    Button,
    Label,
    DrawingArea,
    Scale,
    Scrollable
}

///
/// Actions that can be performed on a widget
/// 
#[derive(Clone)]
pub enum GtkWidgetAction {
    /// Creates a new widget of the specifed type
    New(GtkWidgetType),

    /// Deletes this widget (and any child widgets it may contain)
    Delete
}

///
/// GTK actions that can be requested
/// 
#[derive(Clone)]
pub enum GtkAction {
    /// Shuts down Gtk
    Stop,

    /// Performs some actions on a window
    Window(WindowId, Vec<GtkWindowAction>),

    /// Performs some actions on a widget
    Widget(WidgetId, Vec<GtkWidgetAction>)
}
