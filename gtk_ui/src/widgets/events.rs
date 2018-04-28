use glib::object::Downcast;
use glib::translate::*;
use gdk;
use gdk_sys;

use std::mem;

///
/// Returns the GDK device that generated a particular event
/// 
pub fn device_for_event(event: &gdk::Event) -> gdk::Device {
    let device: gdk::Device = unsafe {
        let sys_device: *const gdk_sys::GdkEvent = mem::transmute(event.as_ref());
        let sys_device = gdk_sys::gdk_event_get_source_device(sys_device);
        gdk::Device::from_glib_borrow(sys_device).downcast_unchecked()
    };

    device
}
