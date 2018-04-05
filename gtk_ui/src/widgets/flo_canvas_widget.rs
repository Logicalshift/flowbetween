use super::widget::*;
use super::basic_widget::*;
use super::super::canvas::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_canvas::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// A Flo widget used to draw canvas actions
/// 
pub struct FloDrawingWidget {
    /// The ID of this widget
    widget_id: WidgetId,
    
    /// Drawing actions for this widget (used as a cache for when we need to redraw the pixbufs)
    canvas: Rc<RefCell<Canvas>>,

    /// The pixel buffers containing the rasterized version of the canvas
    pixbufs: Rc<RefCell<PixBufCanvas>>,

    /// The drawing area used by this widget
    drawing_area: gtk::DrawingArea,

    /// This widget represented as a widget
    as_widget: gtk::Widget
}


impl FloDrawingWidget {
    ///
    /// Retrieves the viewport for a canvas
    /// 
    fn get_viewport(drawing_area: gtk::DrawingArea) -> CanvasViewport {
        unimplemented!()
    }

    ///
    /// Creates a new drawing widget
    /// 
    pub fn new(widget_id: WidgetId, drawing_area: gtk::DrawingArea) -> FloDrawingWidget {
        let canvas      = Canvas::new();
        let as_widget   = drawing_area.clone().upcast::<gtk::Widget>();
        let pixbufs     = PixBufCanvas::new(CanvasViewport::minimal());

        let canvas      = Rc::new(RefCell::new(canvas));
        let pixbufs     = Rc::new(RefCell::new(pixbufs));

        FloDrawingWidget {
            widget_id:      widget_id,
            canvas:         canvas,
            pixbufs:        pixbufs,
            drawing_area:   drawing_area,
            as_widget:      as_widget
        }
    }
}

impl GtkUiWidget for FloDrawingWidget {
    fn id(&self) -> WidgetId {
        self.widget_id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            other_action    => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Drawing areas can't have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
