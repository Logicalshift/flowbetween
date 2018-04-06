use super::widget::*;
use super::widget_data::*;
use super::super::gtk_thread::*;
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
    /// Where the paint events should be sent to
    event_sink: GtkEventSink
}

///
/// Provides support for the painting events for a widget
/// 
pub struct PaintActions {
    /// Mutable core of the paint actions
    core: RefCell<PaintCore>
}

impl PaintActions {
    /// 
    /// Creates new paint data
    /// 
    fn new(event_sink: GtkEventSink) -> PaintActions {
        let core = PaintCore { 
            event_sink: event_sink
        };

        PaintActions {
            core: RefCell::new(core)
        }
    }

    ///
    /// Wires an existing widget for paint events
    /// 
    pub fn wire_widget<W: GtkUiWidget>(widget_data: &WidgetData, event_sink: RefCell<GtkEventSink>, widget: &W, device: GtkPaintDevice) {
        let widget_id       = widget.id();
        let existing_wiring = widget_data.get_widget_data::<PaintActions>(widget_id);

        match existing_wiring {
            Some(paint) => {
                // TODO: Add the device to the set already in use
            },

            None => {
                // Create some new wiring
                widget_data.set_widget_data(widget_id, PaintActions::new(event_sink.into_inner()));

                // Fetch the wiring
                let new_wiring = widget_data.get_widget_data::<PaintActions>(widget_id).unwrap();

                // Connect the paint events to this widget
                Self::connect_events(widget.get_underlying(), Rc::clone(&*new_wiring));

                // TODO: add this device to the set supported by this widget
            }
        }
    }

    ///
    /// Connects paint events to a GTK widget
    /// 
    fn connect_events(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {

    }
}