use super::widget::*;
use super::widget_data::*;
use super::flo_fixed_widget::*;
use super::flo_canvas_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk;
use gtk::prelude::*;
use cairo;

use std::rc::*;
use std::cell::*;

///
/// Represents a canvas widget that is also usable as a drawing area
///
pub struct FloCanvasLayoutWidget {
    /// This widget as a canvas
    as_drawing: FloDrawingWidget,

    /// This widget as a fixed widget
    as_fixed: FloFixedWidget
}

impl FloCanvasLayoutWidget {
    ///
    /// Creates a new drawing widget
    ///
    pub fn new<W: 'static+Clone+Cast+IsA<gtk::Widget>+IsA<gtk::Container>+FixedWidgetLayout>(widget_id: WidgetId, drawing_area: W, data: Rc<WidgetData>) -> FloCanvasLayoutWidget {
        // This takes on some aspects of the drawing widget and some aspects of the fixed widget
        let drawing = FloDrawingWidget::new(widget_id, drawing_area.clone(), Rc::clone(&data));
        let fixed   = FloFixedWidget::new(widget_id, drawing_area, data);

        FloCanvasLayoutWidget {
            as_drawing: drawing,
            as_fixed:   fixed
        }
    }
}

impl GtkUiWidget for FloCanvasLayoutWidget {
    fn id(&self) -> WidgetId {
        self.as_fixed.id()
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        // Drawing actions to the drawing widget, everything else to the fixed widget
        match action {
            &GtkWidgetAction::Content(WidgetContent::Draw(_))   => { self.as_drawing.process(flo_gtk, action); },
            other_action                                        => { self.as_fixed.process(flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Act as a layout widget most of the time
        self.as_fixed.set_children(children)
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        // Both widget behaviours are the same here
        self.as_fixed.get_underlying()
    }

    fn draw_manual(&self, context: &cairo::Context) {
        self.as_drawing.draw_manual(context);
    }
}
