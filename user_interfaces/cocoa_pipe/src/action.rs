use flo_ui::*;
use flo_canvas::{Draw, Color};

use super::view_type::*;

///
/// Represents a property binding in a Cocoa application
///
#[derive(Clone, PartialEq, Debug)]
pub enum AppProperty {
    Nothing,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),

    /// Property is bound to a property ID in the view model
    Bind(usize, usize)
}

///
/// Represents a position coordinate
///
#[derive(Clone, PartialEq, Debug)]
pub enum AppPosition {
    At(f64),
    Floating(AppProperty, f64),
    Offset(f64),
    Stretch(f64),
    Start,
    End,
    After
}

///
/// Represents the bounds of a particular control
///
#[derive(Clone, PartialEq, Debug)]
pub struct AppBounds {
    pub x1: AppPosition,
    pub y1: AppPosition,
    pub x2: AppPosition,
    pub y2: AppPosition
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

    /// Requests a tick next time the user is given control of the application
    RequestTick,
}

///
/// Enumeration of possible actions that can be performed by a Cocoa View
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewAction {
    /// Requests a particular event type from this view
    RequestEvent(ViewEvent, String),

    /// Removes the view from its superview
    RemoveFromSuperview,

    /// Adds the view with the specified ID as a subview of this view
    AddSubView(usize),

    /// Inserts a new subview before an existing subview. The first argument is the view ID, the second is the index to add it at
    InsertSubView(usize, usize),

    /// Sets the bounds of the view for layout
    SetBounds(AppBounds),

    /// Sets the Z-Index of the view
    SetZIndex(f64),

    /// Sets the colour of any text or similar element the view might contain
    SetForegroundColor(Color),

    /// Sets the background colour of this view
    SetBackgroundColor(Color),

    /// Sets the text to display in a control
    SetText(AppProperty),

    /// Sets the image to display in a control
    SetImage(Resource<Image>),

    /// Sets the font size in pixels
    SetFontSize(f64),

    /// Sets the text alignment
    SetTextAlignment(TextAlign),

    /// Sets the font weight
    SetFontWeight(f64),

    /// Sets the minimum size of the scroll area (if the view is a scrolling type)
    SetScrollMinimumSize(f64, f64),

    /// Specifies the visibility of the horizontal scroll bar
    SetHorizontalScrollBar(ScrollBarVisibility),

    /// Specifies the visibility of the vertical scroll bar
    SetVerticalScrollBar(ScrollBarVisibility),

    /// Specifies the padding around the view
    SetPadding(f64, f64, f64, f64),

    /// Sets the ID for this view
    SetId(String),

    /// Draws on the canvas for this view
    Draw(Vec<Draw>),

    /// Draws on the hardware-accellerated canvas for this view
    DrawGpu(Vec<Draw>),

    /// Sets part of the state of this view
    SetState(ViewStateUpdate),

    /// Performs an action that relates to a pop-up view
    Popup(ViewPopupAction)
}

///
/// Actions that update the state of the view (how it is displayed, or what it is displaying)
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewStateUpdate {
    /// Boolean indicating if this view is selected or not
    Selected(AppProperty),

    /// Boolean indicating if this view has a badge attached to it
    Badged(AppProperty),

    /// Boolean indicating if this view is enabled for interaction or greyed out
    Enabled(AppProperty),

    /// Property indicating the value for this view
    Value(AppProperty),

    /// Property indicating the range of valid values for this view
    Range(AppProperty, AppProperty),

    /// If this view is new, the priority with which to steal focus
    FocusPriority(AppProperty),

    /// Prevents this view from moving within a scroll view
    FixScrollAxis(FixedAxis),

    /// Adds a class name to this view (used as a hint to change rendering styles)
    AddClass(String)
}

///
/// Actions relating to a pop-up view
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewPopupAction {
    /// Updates the property that specifies whether or not this popup is on screen or not
    Open(AppProperty),

    /// Sets the direction the popup is offset from the original view
    SetDirection(PopupDirection),

    /// Sets the size of the popup window in pixels
    SetSize(f64, f64),

    /// Sets the number of pixels betweem the popup and the center of the target view
    SetOffset(f64)
}

#[derive(Clone, Copy, PartialEq, Debug, FromPrimitive, ToPrimitive)]
pub enum AppPaintDevice {
    MouseLeft       = 0,
    MouseMiddle     = 1,
    MouseRight      = 2,
    Pen             = 3,
    Eraser          = 4,
    Touch           = 5
}

impl AppPaintDevice {
    ///
    /// Converts an AppPaintDevice into a UI paint device
    ///
    pub fn into_paint_device(&self) -> PaintDevice {
        match self {
            AppPaintDevice::MouseLeft   => PaintDevice::Mouse(MouseButton::Left),
            AppPaintDevice::MouseMiddle => PaintDevice::Mouse(MouseButton::Middle),
            AppPaintDevice::MouseRight  => PaintDevice::Mouse(MouseButton::Right),
            AppPaintDevice::Pen         => PaintDevice::Pen,
            AppPaintDevice::Eraser      => PaintDevice::Eraser,
            AppPaintDevice::Touch       => PaintDevice::Touch
        }
    }
}

///
/// Events that can be requested from a view
///
#[derive(Clone, PartialEq, Debug)]
pub enum ViewEvent {
    /// User has clicked the control contained within this view
    Click,

    /// Send event when the user clicks in a view that's not either this view or a subview
    Dismiss,

    /// Send events when the view is scrolled, indicating which area is visible
    VirtualScroll(f64, f64),

    /// Send events for painting actions on this view
    Paint(AppPaintDevice),

    /// Send actions for dragging this view
    Drag,

    /// Send events when this view is focused
    Focused,

    /// Event sent when the value is being changed
    EditValue,

    /// Event sent when the value has been edited
    SetValue,

    /// Event sent when some EditValues were sent but the editing was cancelled
    CancelEdit
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
