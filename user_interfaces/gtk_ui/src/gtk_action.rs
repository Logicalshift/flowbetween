use super::gtk_widget_event_type::*;

use flo_ui as ui;
use flo_canvas as canvas;

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
    ToggleButton,
    CheckBox,
    TextBox,
    Label,
    Scale,
    ScrollArea,
    Popover,

    Overlay,

    Rotor,
    CanvasDrawingArea,
    CanvasLayout,
    CanvasRender
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

    /// Marks this widget as displayed
    Show,

    /// Put this widget inside an event box
    IntoEventBox,

    /// Updates the layout of this widget
    Layout(WidgetLayout),

    /// Updates the content of this widget
    Content(WidgetContent),

    /// Updates the appearance of this widget
    Appearance(ui::Appearance),

    /// Updates the state of this widget
    State(WidgetState),

    /// Updates the font properties for this widget
    Font(ui::Font),

    /// Updates how the content of this widget scrolls
    Scroll(ui::Scroll),

    /// Controls the popup attributes of this widget
    Popup(WidgetPopup),

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

    /// Specifies a drawing to perform on this widget
    Draw(Vec<canvas::Draw>)
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
    BoundingBox(ui::Bounds),

    /// Specifies the floating offset for this widget
    Floating(f64, f64),

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

    /// Sets whether or not this widget is enabled
    SetEnabled(bool),

    /// Sets the value of this widget as a bool
    SetValueBool(bool),

    /// Sets the value of this widget
    SetValueFloat(f64),

    /// Sets the value of this widget as an integer
    SetValueInt(i64),

    /// Sets the value of this widget as a text string
    SetValueText(String),

    /// Sets the minimum value for this widget
    SetRangeMin(f64),

    /// Sets the maximum value for this widget
    SetRangeMax(f64)
}

impl From<WidgetState> for GtkWidgetAction {
    fn from(item: WidgetState) -> GtkWidgetAction {
        GtkWidgetAction::State(item)
    }
}

///
/// Actions for widgets supporting pop-up behaviour
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WidgetPopup {
    /// Sets the direction this popup will open
    SetDirection(ui::PopupDirection),

    /// Sets the size of this popup
    SetSize(u32, u32),

    /// Sets the offset of this popup from the center of its parent widget
    SetOffset(u32),

    /// Sets whether or not this popup is open
    SetOpen(bool)
}

impl From<WidgetPopup> for GtkWidgetAction {
    fn from (item: WidgetPopup) -> GtkWidgetAction {
        GtkWidgetAction::Popup(item)
    }
}

impl From<ui::Font> for GtkWidgetAction {
    fn from(item: ui::Font) -> GtkWidgetAction {
        GtkWidgetAction::Font(item)
    }
}

impl From<ui::Appearance> for GtkWidgetAction {
    fn from(item: ui::Appearance) -> GtkWidgetAction {
        GtkWidgetAction::Appearance(item)
    }
}

impl From<ui::Scroll> for GtkWidgetAction {
    fn from(item: ui::Scroll) -> GtkWidgetAction {
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
