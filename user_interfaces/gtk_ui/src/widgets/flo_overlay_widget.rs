use super::image::*;
use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Handles events for 'overlay' widgets that make it possible to draw widgets on top of each other
///
/// This is useful for say creating an image behind a collection widget. All other controls are laid
/// out over the top of the first control.
///
pub struct FloOverlayWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The overlay widget that this is managing
    overlay: gtk::Overlay,

    /// The overlay again, but cast to a widget
    as_widget: gtk::Widget,

    /// The IDs of the child widgets of this widget
    child_ids: Vec<WidgetId>,

    /// The widget data
    widget_data: Rc<WidgetData>
}

impl FloOverlayWidget {
    ///
    /// Creates a new overlay widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::Overlay>+IsA<gtk::Widget>>(id: WidgetId, overlay_widget: W, widget_data: Rc<WidgetData>) -> FloOverlayWidget {
        FloOverlayWidget {
            id:             id,
            overlay:        overlay_widget.clone().upcast::<gtk::Overlay>(),
            as_widget:      overlay_widget.upcast::<gtk::Widget>(),
            child_ids:      vec![],
            widget_data:    widget_data
        }
    }
}

impl GtkUiWidget for FloOverlayWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;

        match action {
            _                       => {
                process_basic_widget_action(self, flo_gtk, action);
            }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        let widget_data = &self.widget_data;
        let container   = &self.overlay;

        // Remove any child widgets added by the previous call to this function
        self.child_ids.drain(..)
            .map(|child_id| widget_data.get_widget(child_id))
            .for_each(|widget| { widget.map(|widget| container.remove(widget.borrow().get_underlying())); });

        // Add children to this widget
        self.child_ids.extend(children.iter().map(|child_widget| child_widget.borrow().id()));
        for child in children.iter() {
            self.overlay.add(child.borrow().get_underlying());
        }
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
