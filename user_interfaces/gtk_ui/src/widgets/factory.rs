use super::widget::*;
use super::flo_bin_widget::*;
use super::flo_fixed_widget::*;
use super::flo_popover_widget::*;
use super::flo_label_widget::*;
use super::flo_scale_widget::*;
use super::flo_rotor_widget::*;
use super::flo_scroll_widget::*;
use super::flo_canvas_widget::*;
use super::flo_overlay_widget::*;
use super::flo_textbox_widget::*;
use super::flo_checkbox_widget::*;
use super::flo_render_canvas_widget::*;
use super::flo_canvas_layout_widget::*;
use super::widget_data::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;

///
/// Constructs a new widget of the specified type
///
pub fn create_widget(id: WidgetId, widget_type: GtkWidgetType, widget_data: Rc<WidgetData>) -> Box<dyn GtkUiWidget> {
    use self::GtkWidgetType::*;

    match widget_type {
        Generic             => Box::new(FloFixedWidget::new(id, gtk::Fixed::new(), widget_data)),
        Layout              => {
            let no_adjustment: Option<gtk::Adjustment> = None;
            Box::new(FloFixedWidget::new(id, gtk::Layout::new(no_adjustment.as_ref(), no_adjustment.as_ref()), widget_data))
        },
        Fixed               => Box::new(FloFixedWidget::new(id, gtk::Fixed::new(), widget_data)),
        Button              => Box::new(FloBinWidget::new(id, gtk::Button::new(), widget_data)),
        ToggleButton        => Box::new(FloBinWidget::new(id, gtk::ToggleButton::new(), widget_data)),
        CheckBox            => Box::new(FloCheckBoxWidget::new(id, gtk::CheckButton::new())),
        TextBox             => Box::new(FloTextBoxWidget::new(id, gtk::Entry::new())),
        Label               => Box::new(FloLabelWidget::new(id, gtk::Label::new(None))),
        Popover             => Box::new(FloPopoverWidget::new(id, gtk::Layout::new::<gtk::Adjustment, gtk::Adjustment>(None, None), widget_data)),

        Overlay             => Box::new(FloOverlayWidget::new(id, gtk::Overlay::new(), gtk::Layout::new::<gtk::Adjustment, gtk::Adjustment>(None, None), widget_data)),

        ScrollArea          => {
            let no_adjustment: Option<gtk::Adjustment> = None;
            Box::new(FloScrollWidget::new(id, gtk::ScrolledWindow::new(no_adjustment.as_ref(), no_adjustment.as_ref()), widget_data))
        },
        Rotor               => Box::new(FloRotorWidget::new(id, gtk::DrawingArea::new())),
        CanvasDrawingArea   => Box::new(FloDrawingWidget::new(id, gtk::DrawingArea::new(), widget_data)),
        CanvasLayout        => {
            let no_adjustment: Option<gtk::Adjustment> = None;
            Box::new(FloCanvasLayoutWidget::new(id, gtk::Layout::new(no_adjustment.as_ref(), no_adjustment.as_ref()), widget_data))
        },
        CanvasRender        => Box::new(FloRenderCanvasWidget::new_opengl(id, gtk::GLArea::new(), widget_data)),

        Scale               => {
            let no_adjustment: Option<gtk::Adjustment> = None;
            let scale = gtk::Scale::new(gtk::Orientation::Horizontal, no_adjustment.as_ref());
            scale.set_draw_value(false);
            Box::new(FloScaleWidget::new(id, scale))
        },
    }
}
