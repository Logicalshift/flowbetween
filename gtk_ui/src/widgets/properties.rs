use super::super::gtk_action::*;

use glib;

pub const FLO_WINDOW_ID: &str = "flo_window_id";
pub const FLO_WIDGET_ID: &str = "flo_widget_id";

///
/// Flowbetween properties attached to Gtk widgets
///
pub trait FloProperties {
    /// Sets the window ID for this object
    fn flo_set_window_id(&mut self, new_id: WindowId);
    /// Retrieves the window ID for this object if there is one
    fn flo_get_window_id(&self) -> Option<WindowId>;

    /// Sets the widget ID for this object
    fn flo_set_widget_id(&mut self, new_id: WidgetId);
    /// Retrieves the widget ID for this object if there is one
    fn flo_get_widget_id(&self) -> Option<WidgetId>;
}

///
/// All glib objects get the flo_properties convenience methods
/// 
impl<T: glib::ObjectExt> FloProperties for T {
    fn flo_set_window_id(&mut self, new_id: WindowId) {
        self.set_property(FLO_WINDOW_ID, &glib::AnyValue::new(new_id)).unwrap();
    }

    fn flo_get_window_id(&self) -> Option<WindowId> {
        let value       = self.get_property(FLO_WINDOW_ID).ok();
        let value       = value.as_ref();
        let any_value   = value.and_then(|value| value.get::<&glib::AnyValue>());

        any_value.and_then(|any_value| any_value.downcast_ref::<WindowId>().cloned())
    }

    fn flo_set_widget_id(&mut self, new_id: WidgetId) {
        self.set_property(FLO_WIDGET_ID, &glib::AnyValue::new(new_id)).unwrap();
    }

    fn flo_get_widget_id(&self) -> Option<WidgetId> {
        let value       = self.get_property(FLO_WIDGET_ID).ok();
        let value       = value.as_ref();
        let any_value   = value.and_then(|value| value.get::<&glib::AnyValue>());

        any_value.and_then(|any_value| any_value.downcast_ref::<WidgetId>().cloned())
    }
}