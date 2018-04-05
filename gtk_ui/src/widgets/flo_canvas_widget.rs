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

struct DrawingCore {
    /// Canvas (used to cache the drawing commands used for a redraw)
    canvas:         Canvas,

    /// Raster version of the canvas
    pixbufs:        PixBufCanvas,

    /// Set to true if the canvas needs to be redrawn to the pixbufs
    need_redraw:    bool

}

///
/// A Flo widget used to draw canvas actions
/// 
pub struct FloDrawingWidget {
    /// The ID of this widget
    widget_id: WidgetId,

    /// The drawing area used by this widget
    drawing_area: gtk::DrawingArea,

    /// This widget represented as a widget
    as_widget: gtk::Widget,

    /// Core data cell
    core: Rc<RefCell<DrawingCore>>
}


impl FloDrawingWidget {
    ///
    /// Creates a new drawing widget
    /// 
    pub fn new(widget_id: WidgetId, drawing_area: gtk::DrawingArea) -> FloDrawingWidget {
        // Create the data structures
        let canvas      = Canvas::new();
        let as_widget   = drawing_area.clone().upcast::<gtk::Widget>();
        let pixbufs     = PixBufCanvas::new(CanvasViewport::minimal());

        let core        = DrawingCore {
            canvas:         canvas,
            pixbufs:        pixbufs,
            need_redraw:    false
        };
        let core        = Rc::new(RefCell::new(core));

        // Wire events
        Self::connect_size_allocate(&drawing_area, Rc::clone(&core));

        // Generate the widget
        FloDrawingWidget {
            widget_id:      widget_id,
            drawing_area:   drawing_area,
            as_widget:      as_widget,
            core:           core
        }
    }

    ///
    /// Deals with resizing the drawing area
    /// 
    fn connect_size_allocate(drawing_area: &gtk::DrawingArea, core: Rc<RefCell<DrawingCore>>) {
        drawing_area.connect_size_allocate(move |widget, new_allocation| {
            // Unlock the core
            let mut core = core.borrow_mut();

            // Pixbufs will now be invalid
            core.need_redraw = true;

            // Update the viewport for the pixbufs
            core.pixbufs.set_viewport(Self::get_viewport(widget, new_allocation));
        });
    }

    ///
    /// Retrieves the viewport for a canvas
    /// 
    fn get_viewport(_drawing_area: &gtk::DrawingArea, allocation: &gtk::Allocation) -> CanvasViewport {
        // TODO: search for a containing scrolling area and limit to the displayed size

        CanvasViewport {
            width:              allocation.width.min(1),
            height:             allocation.height.min(1),
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     allocation.width.min(1),
            viewport_height:    allocation.height.min(1)
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
