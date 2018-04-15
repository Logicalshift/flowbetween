use flo_ui::*;

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
            PaintDevice::Mouse(MouseButton::Left)       => GtkPaintDevice::Mouse(0),
            PaintDevice::Mouse(MouseButton::Middle)     => GtkPaintDevice::Mouse(2),
            PaintDevice::Mouse(MouseButton::Right)      => GtkPaintDevice::Mouse(1),
            PaintDevice::Mouse(MouseButton::Other(_))   => GtkPaintDevice::None,
            PaintDevice::Pen                            => GtkPaintDevice::Stylus,
            PaintDevice::Eraser                         => GtkPaintDevice::Eraser,
            PaintDevice::Touch                          => GtkPaintDevice::Touch
        }
    }
}