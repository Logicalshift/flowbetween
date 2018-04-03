use super::widget::*;
use super::flo_fixed_widget::*;
use super::flo_popup_widget::*;
use super::flo_label_widget::*;
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
        Generic         => Box::new(FloFixedWidget::new(id, gtk::Fixed::new(), widget_data)),
        Layout          => Box::new(FloFixedWidget::new(id, gtk::Layout::new(None, None), widget_data)),
        Fixed           => Box::new(FloFixedWidget::new(id, gtk::Fixed::new(), widget_data)),
        Button          => Box::new(BasicWidget::new(id, gtk::ToggleButton::new())),
        Label           => Box::new(FloLabelWidget::new(id, gtk::Label::new(None))),
        DrawingArea     => Box::new(BasicWidget::new(id, gtk::DrawingArea::new())),
        Scale           => Box::new(BasicWidget::new(id, gtk::Scale::new(gtk::Orientation::Horizontal, None))),
        Popup           => Box::new(FloPopupWidget::new(id, gtk::Fixed::new()))
    }
}
