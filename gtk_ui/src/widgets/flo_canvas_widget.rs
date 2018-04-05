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
            need_redraw:    true
        };
        let core        = Rc::new(RefCell::new(core));

        // Test canvas
        core.borrow_mut().canvas.draw(|gc| {
            gc.stroke_color(Color::Rgba(1.0, 0.0, 0.0, 1.0));

            gc.new_path();
            gc.move_to(-1.0, -1.0);
            gc.line_to(1.0, 1.0);
            gc.stroke();

            gc.new_path();
            gc.move_to(1.0, -1.0);
            gc.line_to(-1.0, 1.0);
            gc.stroke();
        });

        // Wire events
        Self::connect_size_allocate(&drawing_area, Rc::clone(&core));
        Self::connect_draw(&drawing_area, Rc::clone(&core));

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

            // Make sure the widget is redrawn
            widget.queue_draw();
        });
    }

    ///
    /// Redraws the core from the canvas
    /// 
    fn redraw(core: &mut DrawingCore) {
        for draw in core.canvas.get_drawing() {
            core.pixbufs.draw(draw);
        }
    }

    ///
    /// Deals with drawing the main drawing area
    /// 
    fn connect_draw(drawing_area: &gtk::DrawingArea, core: Rc<RefCell<DrawingCore>>) {
        drawing_area.connect_draw(move |_widget, context| {
            // Fetch the core
            let mut core = core.borrow_mut();

            // Make sure everything has been drawn up to date
            if core.need_redraw {
                Self::redraw(&mut core);
                core.need_redraw = false;
            }

            // Render the pixbufs
            core.pixbufs.render_to_context(context);

            Inhibit(true)
        });
    }

    ///
    /// Performs some drawing actions on this canvas
    /// 
    fn draw<DrawIter: Send+IntoIterator<Item=Draw>>(&mut self, actions: DrawIter) {
        // Get the core to do drawing on
        let mut core = self.core.borrow_mut();

        if core.need_redraw {
            // Only need to store these actions in the canvas
            core.canvas.write(actions.into_iter().collect());
        } else {
            // Write to the canvas and the core
            let actions: Vec<_> = actions.into_iter().collect();
            for action in actions.iter() {
                core.pixbufs.draw(*action);
            }
            core.canvas.write(actions);
        }

        // Note that a redraw is needed
        self.drawing_area.queue_draw();
    }

    ///
    /// Retrieves the viewport for a canvas
    /// 
    fn get_viewport(_drawing_area: &gtk::DrawingArea, allocation: &gtk::Allocation) -> CanvasViewport {
        // TODO: search for a containing scrolling area and limit to the displayed size

        CanvasViewport {
            width:              allocation.width.max(1),
            height:             allocation.height.max(1),
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     allocation.width.max(1),
            viewport_height:    allocation.height.max(1)
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
