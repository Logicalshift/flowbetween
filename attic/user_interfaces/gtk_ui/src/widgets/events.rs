use flo_ui::*;

use glib::translate::*;
use gdk;
use gdk_sys;

use std::mem;

///
/// Returns a paint device corresponding to a GDK input source
///
pub fn paint_device_for_source(source: gdk::InputSource) -> PaintDevice {
    use gdk::InputSource::*;

    match source {
        Mouse           => PaintDevice::Mouse(MouseButton::Left),
        Cursor          => PaintDevice::Mouse(MouseButton::Left),
        Keyboard        => PaintDevice::Mouse(MouseButton::Left),
        Touchpad        => PaintDevice::Mouse(MouseButton::Left),
        Trackpoint      => PaintDevice::Mouse(MouseButton::Left),
        __Unknown(_)    => PaintDevice::Mouse(MouseButton::Left),
        Pen             => PaintDevice::Pen,
        Eraser          => PaintDevice::Eraser,
        Touchscreen     => PaintDevice::Touch,
        TabletPad       => PaintDevice::Pen,

        _               => PaintDevice::Mouse(MouseButton::Left)
    }
}

///
/// Returns the GDK device that generated a particular event
///
pub fn device_for_event(event: &gdk::Event) -> Borrowed<gdk::Device> {
    let device = unsafe {
        let sys_device: *const gdk_sys::GdkEvent = mem::transmute(event.as_ref());
        let sys_device = gdk_sys::gdk_event_get_source_device(sys_device);
        gdk::Device::from_glib_borrow(sys_device)
    };

    device
}
