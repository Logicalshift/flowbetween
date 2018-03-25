use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_action::*;

use gtk;

///
/// Constructs a new widget of the specified type
/// 
pub fn create_widget(id: WidgetId, widget_type: GtkWidgetType) -> Box<GtkUiWidget> {
    use self::GtkWidgetType::*;

    match widget_type {
        Generic         => Box::new(BasicWidget::new(id, gtk::Layout::new(None, None))),
        Layout          => Box::new(BasicWidget::new(id, gtk::Layout::new(None, None))),
        Fixed           => Box::new(BasicWidget::new(id, gtk::Fixed::new())),
        Button          => Box::new(BasicWidget::new(id, gtk::Button::new())),
        Label           => Box::new(BasicWidget::new(id, gtk::Label::new(None))),
        DrawingArea     => Box::new(BasicWidget::new(id, gtk::DrawingArea::new())),
        Scale           => Box::new(BasicWidget::new(id, gtk::Scale::new(gtk::Orientation::Horizontal, None)))
    }
}
