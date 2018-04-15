use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_bin_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

// TODO: actually manage popups

///
/// The popup widget is used to manage GTK popup widgets
/// 
pub struct FloPopoverWidget {
    /// The ID of the widget
    id: WidgetId,

    /// The popup is a bin widget, so we delegate most of the actions there
    bin: FloBinWidget,

    /// THe popover widget
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
    pub fn new<Src: Clone+Cast+IsA<gtk::Popover>>(id: WidgetId, widget: Src, widget_data: Rc<WidgetData>) -> FloPopoverWidget {
        let popover = widget.clone().upcast::<gtk::Popover>();
        let bin     = popover.clone().upcast::<gtk::Bin>();
        let widget  = popover.clone().upcast::<gtk::Widget>();

        let bin     = FloBinWidget::new(id, bin, widget_data);

        FloPopoverWidget {
            id:         id,
            bin:        bin,
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
            &Popup(SetSize(width, height))  => { self.popover.size_allocate(&mut gtk::Rectangle { x: 0, y: 0, width: width as i32, height: height as i32 }) },
            &Popup(SetOffset(offset))       => { self.offset = offset },

            &Popup(SetOpen(is_open))        => { 
                self.popover.set_relative_to(&self.popover.get_parent().unwrap());

                if is_open {
                    self.popover.show();
                } else {
                    self.popover.hide();
                }
            },

            // Everything else is processed by the bin widget
            other_action    => { self.bin.process(flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Pass on to the bin widget
        self.bin.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}
