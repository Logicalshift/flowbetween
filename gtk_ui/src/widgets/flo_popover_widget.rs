use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

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

    /// Offset used for positioning
    offset: u32
}

impl FloPopoverWidget {
    ///
    /// Creates a basic widget
    /// 
    pub fn new<Src: Clone+Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src, widget_data: Rc<WidgetData>) -> FloPopoverWidget {
        let widget  = widget.upcast::<gtk::Widget>();
        let popover = gtk::Popover::new(&widget);
        let content = gtk::Fixed::new();

        // Content widget used to contain the content for this popover
        popover.add(&content);

        content.set_size_request(100, 100);
        content.size_allocate(&mut gtk::Rectangle { x: 0, y: 0, width: 100, height: 100 });

        let content = FloFixedWidget::new(id, content, Rc::clone(&widget_data));

        FloPopoverWidget {
            id:         id,
            content:    content,
            popover:    popover,
            widget:     widget,
            offset:     0
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
            &Popup(SetDirection(direction)) => { self.popover.set_position(Self::position_for_direction(direction)); },
            &Popup(SetSize(width, height))  => { self.content.get_underlying().set_size_request(width as i32, height as i32); },
            &Popup(SetOffset(offset))       => { self.offset = offset },

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
