use flo_ui::*;

use gdk;

///
/// Device used for painting
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GtkPaintDevice {
    None,
    Mouse(i32),
    Touch,
    Stylus,
    Eraser
}

impl From<GtkPaintDevice> for Vec<gdk::InputSource> {
    fn from(device: GtkPaintDevice) -> Vec<gdk::InputSource> {
        use self::GtkPaintDevice::*;

        match device {
            None        => vec![],
            Mouse(_)    => vec![gdk::InputSource::Mouse, gdk::InputSource::Trackpoint, gdk::InputSource::Cursor, gdk::InputSource::Touchpad],
            Touch       => vec![gdk::InputSource::Touchscreen],
            Stylus      => vec![gdk::InputSource::Pen, gdk::InputSource::TabletPad],
            Eraser      => vec![gdk::InputSource::Eraser]
        }
    }
}

impl GtkPaintDevice {
    ///
    /// Returns a list of mouse buttons that a particular GTK device should respond to (the empty list indicates all buttons for this input source)
    ///
    pub fn buttons(&self) -> Vec<u32> {
        use self::GtkPaintDevice::*;

        match self {
            &None | &Touch | &Stylus | &Eraser  => vec![],
            &Mouse(ref button)                  => vec![*button as u32]
        }
    }
}

///
/// Types of widget event that can be registered
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GtkWidgetEventType {
    /// User pressed and released the mouse over a particular widget (or any of its children, if they do not generate their own event for this situation)
    Click,

    /// User performed painting actions over a widget
    Paint(GtkPaintDevice),

    /// User dragged the control
    Drag,

    /// User is in the process of editing a value
    EditValue,

    /// User has picked a final value
    SetValue,

    /// Performs virtual scrolling using a grid with the specified width and height
    VirtualScroll(f32, f32),

    /// User has interacted outside of this widget
    Dismiss
}

impl From<PaintDevice> for GtkPaintDevice {
    fn from(device: PaintDevice) -> GtkPaintDevice {
        match device {
            PaintDevice::Other                          => GtkPaintDevice::None,
            PaintDevice::Mouse(MouseButton::Left)       => GtkPaintDevice::Mouse(1),
            PaintDevice::Mouse(MouseButton::Middle)     => GtkPaintDevice::Mouse(2),
            PaintDevice::Mouse(MouseButton::Right)      => GtkPaintDevice::Mouse(3),
            PaintDevice::Mouse(MouseButton::Other(_))   => GtkPaintDevice::None,
            PaintDevice::Pen                            => GtkPaintDevice::Stylus,
            PaintDevice::Eraser                         => GtkPaintDevice::Eraser,
            PaintDevice::Touch                          => GtkPaintDevice::Touch
        }
    }
}
