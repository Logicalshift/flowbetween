use super::gtk_widget_event_type::*;

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
#[derive(Clone, PartialEq, Debug)]
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
    Scale,
    Popup
}

///
/// Actions that can be performed on a widget
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum GtkWidgetAction {
    /// Creates a new widget of the specifed type
    New(GtkWidgetType),

    /// Requests a particular event type for this widget, generating the specified action name
    RequestEvent(GtkWidgetEventType, String),

    /// Removes all the widgets from the specified window and makes this one the new root
    SetRoot(WindowId),

    /// Put this widget inside an event box
    Box,

    /// Updates the layout of this widget
    Layout(WidgetLayout),

    /// Updates the content of this widget
    Content(WidgetContent),

    /// Updates the appearance of this widget
    Appearance(Appearance),

    /// Updates the state of this widget
    State(WidgetState),

    /// Updates the font properties for this widget
    Font(Font),

    /// Updates how the content of this widget scrolls
    Scroll(Scroll),

    /// Deletes this widget (and any child widgets it may contain)
    Delete
}

///
/// Specifies a change to the content of a widget
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum WidgetContent {
    /// Sets the children of this widget to be a particular set of widgets
    SetChildren(Vec<WidgetId>),

    /// Sets the text of this widget to the specified string
    SetText(String),

    /// Adds a class to this widget
    AddClass(String),

    /// Removes a class from this widget
    RemoveClass(String),

    /// Specifies that this widget should draw itself from the specified canvas
    Draw(Resource<Canvas>)
}

impl From<WidgetContent> for GtkWidgetAction {
    fn from(item: WidgetContent) -> GtkWidgetAction {
        GtkWidgetAction::Content(item)
    }
}

///
/// Specifies a change to how a widget is laid out
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum WidgetLayout {
    /// Specifies how this widget should be laid out
    BoundingBox(Bounds),

    /// Specifies the Z-index of this widget
    ZIndex(u32),

    /// Specifies the padding for this widget
    Padding((u32, u32), (u32, u32))
}

impl From<WidgetLayout> for GtkWidgetAction {
    fn from(item: WidgetLayout) -> GtkWidgetAction {
        GtkWidgetAction::Layout(item)
    }
}

///
/// Specifies a change to the state of a widget
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum WidgetState {
    /// Sets whether or not this widget is highlighted as being selected
    SetSelected(bool),

    /// Sets whether or not this widget shows a badge next to it
    SetBadged(bool),

    /// Sets the value of this widget
    SetValueFloat(f32),

    /// Sets the minimum value for this widget
    SetRangeMin(f32),

    /// Sets the maximum value for this widget
    SetRangeMax(f32)
}

impl From<WidgetState> for GtkWidgetAction {
    fn from(item: WidgetState) -> GtkWidgetAction {
        GtkWidgetAction::State(item)
    }
}

impl From<Font> for GtkWidgetAction {
    fn from(item: Font) -> GtkWidgetAction {
        GtkWidgetAction::Font(item)
    }
}

impl From<Appearance> for GtkWidgetAction {
    fn from(item: Appearance) -> GtkWidgetAction {
        GtkWidgetAction::Appearance(item)
    }
}

impl From<Scroll> for GtkWidgetAction {
    fn from(item: Scroll) -> GtkWidgetAction {
        GtkWidgetAction::Scroll(item)
    }
}

///
/// GTK actions that can be requested
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum GtkAction {
    /// Shuts down Gtk
    Stop,

    /// Performs some actions on a window
    Window(WindowId, Vec<GtkWindowAction>),

    /// Performs some actions on a widget
    Widget(WidgetId, Vec<GtkWidgetAction>)
}

impl GtkAction {
    ///
    /// True if this action is a no-op (can be removed from the actions list)
    /// 
    pub fn is_no_op(&self) -> bool {
        match self {
            &GtkAction::Window(_, ref window_actions)   => window_actions.len() == 0,
            &GtkAction::Widget(_, ref widget_actions)   => widget_actions.len() == 0,
            _                                           => false
        }
    }
}