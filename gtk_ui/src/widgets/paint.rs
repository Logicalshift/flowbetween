use super::widget::*;
use super::widget_data::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Contents of the cell that tracks painting state
/// 
struct PaintCore {

}

///
/// Provides support for the painting events for a widget
/// 
pub struct Paint {
    core: RefCell<PaintCore>
}

impl Paint {
    /// 
    /// Creates new paint data
    /// 
    fn new() -> Paint {
        let core = PaintCore { };

        Paint {
            core: RefCell::new(core)
        }
    }

    ///
    /// Wires an existing widget for paint events
    /// 
    pub fn wire_widget<W: GtkUiWidget>(widget_data: &mut WidgetData, widget: &W, device: GtkPaintDevice) {
        let widget_id       = widget.id();
        let existing_wiring = widget_data.get_widget_data::<Paint>(widget_id);

        match existing_wiring {
            Some(paint) => {
                // TODO: Add the device to the set already in use
            },

            None => {
                // Create some new wiring
                widget_data.set_widget_data(widget_id, Paint::new());

                // Fetch the wiring
                let new_wiring = widget_data.get_widget_data::<Paint>(widget_id).unwrap();

                // Connect the paint events to this widget
                Self::connect_events(widget.get_underlying(), Rc::clone(&*new_wiring));

                // TODO: add this device to the set supported by this widget
            }
        }
    }

    ///
    /// Connects paint events to a GTK widget
    /// 
    fn connect_events(widget: &gtk::Widget, paint: Rc<RefCell<Paint>>) {

    }
}