use super::widget::*;
use super::flo_widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::super::gtk_action::*;

use gtk;

use std::rc::*;

///
/// Constructs a new widget of the specified type
/// 
pub fn create_widget(id: WidgetId, widget_type: GtkWidgetType, widget_data: Rc<WidgetData>) -> Box<GtkUiWidget> {
    use self::GtkWidgetType::*;

    match widget_type {
        Generic         => Box::new(FloWidget::new(id, widget_data)),
        Layout          => Box::new(BasicWidget::new(id, gtk::Layout::new(None, None))),
        Fixed           => Box::new(BasicWidget::new(id, gtk::Fixed::new())),
        Button          => Box::new(BasicWidget::new(id, gtk::Button::new())),
        Label           => Box::new(BasicWidget::new(id, gtk::Label::new(None))),
        DrawingArea     => Box::new(BasicWidget::new(id, gtk::DrawingArea::new())),
        Scale           => Box::new(BasicWidget::new(id, gtk::Scale::new(gtk::Orientation::Horizontal, None)))
    }
}
