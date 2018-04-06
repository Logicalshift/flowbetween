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

    /// The scale factor the pixbufs were created at
    scale_factor:   i32,

    /// Set to true if the canvas needs to be redrawn to the pixbufs
    need_redraw:    bool,

    /// Set to true if the size has changed and we need to consider a redraw
    check_size:     bool,

    /// True if a redraw request is pending
    draw_pending: bool
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
            scale_factor:   1,
            need_redraw:    true,
            check_size:     true,
            draw_pending:   false
        };
        let core        = Rc::new(RefCell::new(core));

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

            let new_viewport        = Self::get_viewport(widget, new_allocation);
            let existing_viewport   = core.pixbufs.get_viewport();

            // Cause a redraw if the viewport has changed
            if new_viewport != existing_viewport {
                // Next time we do a draw, we need to check the size
                core.check_size = true;

                // Make sure the widget is redrawn
                Self::queue_draw(&mut *core, widget);
            }
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
        drawing_area.connect_draw(move |widget, context| {
            // Fetch the core
            let mut core = core.borrow_mut();

            // Drawing request is no longer pending
            core.draw_pending = false;

            // If we need to check the size, do so and maybe set the redraw bit
            if core.check_size {
                // Get the current viewport
                let existing_viewport   = core.pixbufs.get_viewport();

                // Get the allocated viewport
                let allocation          = widget.get_allocation();
                let current_viewport    = Self::get_viewport(widget, &allocation);

                // A redraw is required if the size is different from the last time we drew the pixbufs
                if existing_viewport != current_viewport {
                    // Update the viewport
                    core.pixbufs.set_viewport(current_viewport);
                    core.need_redraw = true;
                }

                // Store the scaling factor for the widget
                core.scale_factor = widget.get_scale_factor();

                // Size is checked
                core.check_size = false;
            }

            // Make sure everything has been drawn up to date
            if core.need_redraw {
                Self::redraw(&mut core);
                core.need_redraw = false;
            }

            // Render the pixbufs
            let scale_factor = core.scale_factor;
            let scale_factor = 1.0/(scale_factor as f64);
            context.scale(scale_factor, scale_factor);
            core.pixbufs.render_to_context(context);

            Inhibit(true)
        });
    }

    ///
    /// Indicates a drawing request is required
    /// 
    fn queue_draw(core: &mut DrawingCore, drawing_area: &gtk::DrawingArea) {
        if !core.draw_pending {
            core.draw_pending = true;

            drawing_area.add_tick_callback(|widget: &gtk::DrawingArea, _clock| { widget.queue_draw(); Continue(false) });
        }
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
        Self::queue_draw(&mut *core, &self.drawing_area);
    }

    ///
    /// Retrieves the viewport for a canvas
    /// 
    fn get_viewport(drawing_area: &gtk::DrawingArea, allocation: &gtk::Allocation) -> CanvasViewport {
        // TODO: search for a containing scrolling area and limit to the displayed size

        let scale_factor = drawing_area.get_scale_factor();

        CanvasViewport {
            width:              allocation.width.max(1) * scale_factor,
            height:             allocation.height.max(1) * scale_factor,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     allocation.width.max(1) * scale_factor,
            viewport_height:    allocation.height.max(1) * scale_factor
        }
    }
}

impl GtkUiWidget for FloDrawingWidget {
    fn id(&self) -> WidgetId {
        self.widget_id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            &GtkWidgetAction::Content(WidgetContent::Draw(ref drawing)) => self.draw(drawing.iter().map(|draw| *draw)),
            other_action                                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Drawing areas can't have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
