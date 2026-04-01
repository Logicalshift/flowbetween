use super::widget::*;
use super::widget_data::*;
use super::scroll_size::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_event::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;
use gdk;

use std::rc::*;
use std::cell::*;

///
/// Data used with a popover widget
///
struct FloPopoverData {
    /// Direction the popup will open in
    direction: PopupDirection,

    /// The offset for this popup
    offset: u32,

    /// Set to true if this popover should be considered open
    is_open: bool,

    /// Set to true while re-opening (prevents the popover from immediately closing during a hierarchy change)
    reopening: bool
}

///
/// The popup widget is used to manage GTK popup widgets
///
pub struct FloPopoverWidget {
    /// The ID of the widget
    id: WidgetId,

    /// The content of the pop-over
    content: FloFixedWidget,

    /// The popover widget itself
    popover: gtk::Popover,

    /// The base widget, used as the target for the popup
    widget: gtk::Widget,

    /// The widget data object
    widget_data: Rc<WidgetData>,

    /// Data shared with the layout routine
    popup_data: Rc<RefCell<FloPopoverData>>
}

impl FloPopoverWidget {
    ///
    /// Creates a basic widget
    ///
    pub fn new<Src: Clone+Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src, widget_data: Rc<WidgetData>) -> FloPopoverWidget {
        // Create the various components
        let widget          = widget.upcast::<gtk::Widget>();
        let popover         = gtk::Popover::new(Some(&widget));
        let content         = gtk::Layout::new::<gtk::Adjustment, gtk::Adjustment>(None, None);

        // Create the layout data
        let popup_data      = Rc::new(RefCell::new(FloPopoverData {
            direction:  PopupDirection::Below,
            offset:     0,
            is_open:    false,
            reopening:  false
        }));

        // Set them up
        popover.set_modal(false);
        popover.add(&content);
        popover.set_transitions_enabled(true);

        Self::connect_position_on_size_allocate(&widget, popover.clone(), Rc::clone(&popup_data));
        Self::connect_reopen(&widget, popover.clone(), Rc::clone(&popup_data));

        // TODO: somehow get the styles to cascade from the parent widget
        popover.override_background_color(gtk::StateFlags::NORMAL, Some(&gdk::RGBA { red: 0.20, green: 0.22, blue: 0.25, alpha: 0.94 }));

        // Default size
        content.set_size_request(100, 100);

        // Content widget used to contain the content for this popover
        let content = FloFixedWidget::new(id, content, Rc::clone(&widget_data));

        FloPopoverWidget {
            id:             id,
            content:        content,
            popover:        popover,
            widget:         widget,
            widget_data:    widget_data,
            popup_data:     popup_data
        }
    }

    ///
    /// Re-opens the popover in the event that the base widget changes where it is in the hierarchy
    ///
    /// GTK gets confused about dismissing modal popups when the hierarchy changes while a popup is open.
    ///
    fn connect_reopen(base_widget: &gtk::Widget, popover: gtk::Popover, popover_data: Rc<RefCell<FloPopoverData>>) {
        // Close and re-open the popup if the parent widget moves
        base_widget.connect_hierarchy_changed(move |widget, _| {
            let is_open = popover_data.borrow().is_open;
            if is_open {
                // Mark the widget as re-opening so the hide event doesn't count as dismissing it
                popover_data.borrow_mut().reopening = true;

                // Hide and re-show the popover (GTK gets confused if the pop-over's parent is moved while it's open)
                popover.hide();
                popover_data.borrow_mut().position(&popover, &widget.get_allocation());
                popover.show_all();

                // Reset the reopening flag (next hide will count as a dismissal)
                popover_data.borrow_mut().reopening = false;
            }
        });
    }

    ///
    /// Repositions the popover whenever the base widget's size changes
    ///
    fn connect_position_on_size_allocate(base_widget: &gtk::Widget, popover: gtk::Popover, popover_data: Rc<RefCell<FloPopoverData>>) {
        base_widget.connect_size_allocate(move |_base_widget, allocation| {
            popover_data.borrow().position(&popover, allocation);
        });
    }

    ///
    /// Converts a popup position to a GTK position
    ///
    fn position_for_direction(direction: PopupDirection) -> gtk::PositionType {
        use self::PopupDirection::*;
        use gtk::PositionType;

        match direction {
            OnTop           => PositionType::Bottom,
            WindowCentered  => PositionType::Bottom,
            WindowTop       => PositionType::Bottom,

            Left            => PositionType::Left,
            Right           => PositionType::Right,
            Above           => PositionType::Top,
            Below           => PositionType::Bottom
        }
    }
}

impl FloPopoverData {
    ///
    /// Updates the target position of this popover
    ///
    fn position(&self, popover: &gtk::Popover, target_allocation: &gtk::Rectangle) {
        use self::PopupDirection::*;

        let center_x    = target_allocation.width/4;
        let center_y    = target_allocation.height/4;
        let offset      = self.offset as i32;

        let (pos_x, pos_y) = match self.direction {
            OnTop           => (center_x, center_y),
            WindowCentered  => (center_x, center_y),
            WindowTop       => (center_x, center_y),

            Left            => (center_x - offset, center_y),
            Right           => (center_x + offset, center_y),
            Above           => (center_x, center_y - offset),
            Below           => (center_x, center_y + offset)
        };

        // TODO: point at pos_x, pos_y here (for some reason all my attempts just result in the popup not displaying)
        // Popups seem to have an issue with scaling?
        let pointing_to = gtk::Rectangle { x: center_x-4, y: center_y-4, width: 8, height: 8 };
        popover.set_pointing_to(&pointing_to);
    }
}

impl GtkUiWidget for FloPopoverWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use GtkWidgetAction::{Popup, RequestEvent};
        use WidgetPopup::*;

        match action {
            &Popup(SetDirection(direction)) => { self.popup_data.borrow_mut().direction = direction; self.popover.set_position(Self::position_for_direction(direction)); self.popup_data.borrow().position(&self.popover, &self.widget.get_allocation()); },
            &Popup(SetSize(width, height))  => { 
                self.content.get_underlying().set_size_request(width as i32, height as i32);
                self.widget_data.set_widget_data(self.id, ScrollSize { width: width as i32, height: height as i32 });
            },
            &Popup(SetOffset(offset))       => { self.popup_data.borrow_mut().offset = offset; self.popup_data.borrow().position(&self.popover, &self.widget.get_allocation()); },

            &Popup(SetOpen(is_open))        => {
                // Store whether or not the popover is supposed to be open
                self.popup_data.borrow_mut().is_open = is_open;

                // Open the popup, if the backing widget has a parent
                if self.widget.get_parent().is_some() {
                    if is_open {
                        self.popover.show_all();
                    } else {
                        self.popover.hide();
                    }
                }
            },

            &RequestEvent(GtkWidgetEventType::Dismiss, ref action_name) => {
                let action_name = action_name.clone();
                let sink        = flo_gtk.get_event_sink();
                let widget_id   = self.id;
                let popup_data  = Rc::clone(&self.popup_data);

                // Popover becomes modal again (it needs to be hidden/shown for this to take effect)
                self.popover.hide();
                self.popover.set_modal(true);
                self.popover.show_all();

                // The hide event causes the popup to dismiss
                self.popover.connect_hide(move |_widget| {
                    let reopening = popup_data.borrow().reopening;

                    if !reopening {
                        publish_event(&sink, GtkEvent::Event(widget_id, action_name.clone(), GtkEventParameter::None));
                    }
                });
            },

            // Everything else is processed as if we were a basic widget
            other_action    => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Pass on to the content widget
        self.content.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}
