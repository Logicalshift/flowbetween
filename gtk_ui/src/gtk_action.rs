use flo_ui::*;
use flo_canvas::*;

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
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GtkWidgetType {
    Generic,
    Layout,
    Fixed,
    Button,
    Label,
    DrawingArea,
    Scale
}

///
/// Actions that can be performed on a widget
/// 
#[derive(Clone)]
pub enum GtkWidgetAction {
    /// Creates a new widget of the specifed type
    New(GtkWidgetType),

    /// Removes all the widgets from the specified window and makes this one the new root
    SetRoot(WindowId),

    /// Updates the layout of this widget
    Layout(WidgetLayout),

    /// Updates the content of this widget
    Content(WidgetContent),

    /// Updates the appearance of this widget
    Appearance(Appearance),

    /// Updates the state of this widget
    State(State),

    /// Updates the font properties for this widget
    Font(Font),

    /// Deletes this widget (and any child widgets it may contain)
    Delete
}

///
/// Specifies a change to the content of a widget
/// 
#[derive(Clone)]
pub enum WidgetContent {
    /// Adds this widget as a child of the specified widget
    SetParent(WidgetId),

    /// Sets the text of this widget to the specified string
    SetText(String),

    /// Specifies that this widget should draw itself from the specified canvas
    Draw(Resource<Canvas>)
}

///
/// Specifies a change to how a widget is laid out
/// 
#[derive(Clone)]
pub enum WidgetLayout {
    /// Specifies how this widget should be laid out
    BoundingBox(Bounds),

    /// Specifies the Z-index of this widget
    ZIndex(u32),

    /// Specifies the padding for this widget
    Padding((u32, u32), (u32, u32))
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
