use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

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

    /// The popup widget itself
    widget: gtk::Widget,

    /// Data shared with the layout routine
    data: Rc<RefCell<FloPopoverData>>
}

impl FloPopoverWidget {
    ///
    /// Creates a basic widget
    /// 
    pub fn new<Src: Clone+Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src, widget_data: Rc<WidgetData>) -> FloPopoverWidget {
        // Create the various components
        let widget  = widget.upcast::<gtk::Widget>();
        let popover = gtk::Popover::new(&widget);
        let content = gtk::Fixed::new();

        // Create the layout data
        let data    = Rc::new(RefCell::new(FloPopoverData { direction: PopupDirection::Below, offset: 0 }));

        // Set them up
        popover.set_modal(false);
        popover.add(&content);

        // TODO: somehow get the styles to cascade from the parent widget
        popover.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA { red: 0.20, green: 0.22, blue: 0.25, alpha: 0.94 });

        // Default size
        content.set_size_request(100, 100);

        // Content widget used to contain the content for this popover
        let content = FloFixedWidget::new(id, content, Rc::clone(&widget_data));

        FloPopoverWidget {
            id:         id,
            content:    content,
            popover:    popover,
            widget:     widget,
            data:       data
        }
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

impl GtkUiWidget for FloPopoverWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use GtkWidgetAction::Popup;
        use WidgetPopup::*;

        match action {
            &Popup(SetDirection(direction)) => { self.data.borrow_mut().direction = direction; self.popover.set_position(Self::position_for_direction(direction)); },
            &Popup(SetSize(width, height))  => { self.content.get_underlying().set_size_request(width as i32, height as i32); },
            &Popup(SetOffset(offset))       => { self.data.borrow_mut().offset = offset },

            &Popup(SetOpen(is_open))        => { 
                if is_open {
                    self.popover.show_all();
                } else {
                    self.popover.hide();
                }
            },

            // Everything else is processed as if we were a basic widget
            other_action    => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Pass on to the content widget
        self.content.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}
