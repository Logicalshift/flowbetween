use super::widget::*;
use super::flo_fixed_widget::*;
use super::flo_popup_widget::*;
use super::flo_label_widget::*;
use super::flo_scale_widget::*;
use super::flo_canvas_widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

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
        DrawingArea     => Box::new(FloDrawingWidget::new(id, gtk::DrawingArea::new())),
        Popup           => Box::new(FloPopupWidget::new(id, gtk::Fixed::new())),

        Scale           => {
            let scale = gtk::Scale::new(gtk::Orientation::Horizontal, None);
            scale.set_draw_value(false);
            Box::new(FloScaleWidget::new(id, scale))
        },
    }
}
