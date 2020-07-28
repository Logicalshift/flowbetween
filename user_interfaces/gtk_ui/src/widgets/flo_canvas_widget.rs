use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::super::canvas::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_canvas::*;

use gtk;
use gtk::prelude::*;
use cairo;

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

    /// True if a redraw request is pending
    draw_pending:   bool,

    /// Widget data object
    widget_data:    Rc<WidgetData>
}

///
/// A Flo widget used to draw canvas actions
///
pub struct FloDrawingWidget {
    /// The ID of this widget
    widget_id: WidgetId,

    /// This widget represented as a widget
    as_widget: gtk::Widget,

    /// Core data cell
    core: Rc<RefCell<DrawingCore>>
}

impl FloDrawingWidget {
    ///
    /// Creates a new drawing widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::Widget>>(widget_id: WidgetId, drawing_area: W, data: Rc<WidgetData>) -> FloDrawingWidget {
        // Create the data structures
        let canvas      = Canvas::new();
        let as_widget   = drawing_area.clone().upcast::<gtk::Widget>();
        let pixbufs     = PixBufCanvas::new(CanvasViewport::minimal(), as_widget.get_scale_factor() as f64);

        let core        = DrawingCore {
            canvas:         canvas,
            pixbufs:        pixbufs,
            scale_factor:   1,
            need_redraw:    true,
            draw_pending:   false,
            widget_data:    data
        };
        let core        = Rc::new(RefCell::new(core));

        // Wire events
        Self::connect_size_allocate(&as_widget, Rc::clone(&core));
        Self::connect_draw(&as_widget, Rc::clone(&core));

        // Generate the widget
        FloDrawingWidget {
            widget_id:      widget_id,
            as_widget:      as_widget,
            core:           core
        }
    }

    ///
    /// Deals with resizing the drawing area
    ///
    fn connect_size_allocate(drawing_area: &gtk::Widget, core: Rc<RefCell<DrawingCore>>) {
        drawing_area.connect_size_allocate(move |widget, new_allocation| {
            // Unlock the core
            let mut core = core.borrow_mut();

            let new_viewport        = Self::get_viewport(widget, new_allocation);
            let existing_viewport   = core.pixbufs.get_viewport();

            // Cause a redraw if the viewport has changed
            if new_viewport != existing_viewport {
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
    /// Renders the content of this canvas to a cairo context
    ///
    fn perform_draw(widget: &gtk::Widget, context: &cairo::Context, core: &Rc<RefCell<DrawingCore>>) {
        // Fetch the core
        let mut core = core.borrow_mut();

        // Drawing request is no longer pending
        core.draw_pending = false;

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

            // Store the scaling factor for the widget
            let scale_factor    = widget.get_scale_factor();
            core.scale_factor   = scale_factor;
            core.pixbufs.set_pixel_scale(scale_factor as f64);
        }

        // Make sure everything has been drawn up to date
        if core.need_redraw {
            Self::redraw(&mut core);
            core.need_redraw = false;
        }

        context.save();

        // Resize the context to match the scaling factor
        let scale_factor = core.scale_factor;
        let scale_factor = 1.0/(scale_factor as f64);
        context.scale(scale_factor, scale_factor);

        // Translate according to the viewport
        let viewport = core.pixbufs.get_viewport();
        context.translate(viewport.viewport_x as f64, viewport.viewport_y as f64);

        // Render the pixbufs
        core.pixbufs.render_to_context(context);

        context.restore();
    }

    ///
    /// Deals with drawing the main drawing area
    ///
    fn connect_draw(drawing_area: &gtk::Widget, core: Rc<RefCell<DrawingCore>>) {
        drawing_area.connect_draw(move |widget, context| {
            Self::perform_draw(widget, context, &core);

            Inhibit(true)
        });
    }

    ///
    /// Indicates a drawing request is required
    ///
    fn queue_draw(core: &mut DrawingCore, drawing_area: &gtk::Widget) {
        if !core.draw_pending {
            core.draw_pending = true;

            drawing_area.add_tick_callback(|widget: &gtk::Widget, _clock| { widget.queue_draw(); glib::Continue(false) });
        }
    }

    ///
    /// Updates the matrix used to convert from screen coordinates to canvas coordinates
    ///
    /// (Paint actions in particular should specify canvas coordinates)
    ///
    fn update_translation_matrix(&self, core: &DrawingCore) {
        // Get the transformation matrix: we store this as the translation matrix to use for paint events
        let canvas_to_widget    = core.pixbufs.get_matrix();
        let mut widget_to_canvas = canvas_to_widget;

        // Invert from canvas-to-screen to screen-to-canvas
        widget_to_canvas.invert();

        // Need to remove the viewport translation
        let viewport            = core.pixbufs.get_viewport();
        let mut move_viewport   = cairo::Matrix::identity();
        move_viewport.translate(-(viewport.viewport_x) as f64, -(viewport.viewport_y) as f64);
        widget_to_canvas        = cairo::Matrix::multiply(&move_viewport, &widget_to_canvas);

        if core.scale_factor != 1 {
            // Need to adjust by the scale factor
            let scale_factor = core.scale_factor as f64;
            let mut scale_matrix = cairo::Matrix::identity();
            scale_matrix.scale(scale_factor, scale_factor);

            widget_to_canvas = cairo::Matrix::multiply(&scale_matrix, &widget_to_canvas);
        }

        // Store the transformation matrix for use with generating coordinates for paint events
        core.widget_data.set_widget_data(self.widget_id, widget_to_canvas);
    }

    ///
    /// Performs some drawing actions on this canvas
    ///
    fn draw<DrawIter: Send+IntoIterator<Item=Draw>>(&mut self, actions: DrawIter) {
        // Get the core to do drawing on
        let mut core = self.core.borrow_mut();

        // Make sure the core drawing is up to date
        if core.need_redraw {
            Self::redraw(&mut core);
            core.need_redraw = false;
        }

        // Write to the canvas and the core
        let actions: Vec<_> = actions.into_iter().collect();
        for action in actions.iter() {
            core.pixbufs.draw(*action);
        }
        core.canvas.write(actions);

        // Make sure that the translation matrix is up to date
        self.update_translation_matrix(&*core);

        // Note that a redraw is needed
        Self::queue_draw(&mut *core, &self.as_widget);
    }

    ///
    /// Clips a viewport to only the portion visible in a scrollable area
    ///
    fn clip_viewport_to_scrollable(full_viewport: CanvasViewport, scrollable: &gtk::Scrollable, drawing_area: &gtk::Widget) -> CanvasViewport {
        // Scrollable must also be a widget
        let scrollable_widget = scrollable.clone().dynamic_cast::<gtk::Widget>().unwrap();

        // Will need to scale the coorindates
        let scale       = drawing_area.get_scale_factor();

        // Get the positions for the scrollable
        let hadjust     = scrollable.get_hadjustment().unwrap();
        let vadjust     = scrollable.get_vadjustment().unwrap();

        let hvalue      = hadjust.get_value() as i32;       // = left coordinate
        let hpagesize   = hadjust.get_page_size() as i32;   // = width

        let vvalue      = vadjust.get_value() as i32;       // = top coordinate
        let vpagesize   = vadjust.get_page_size() as i32;   // = height

        // TODO: this should really be '&&', maybe allowing for up to a certain size (we get a giant viewport in the timeline right now, so this isn't done)
        if full_viewport.viewport_width <= hpagesize*scale || full_viewport.viewport_height <= vpagesize*scale {
            // If the scroll region is larger than the viewport then just use the full viewport
            full_viewport
        } else {
            // Turn the values into coorindates on the scrolling area (note that translate_coordinates returns scaled coordinates for some reason)
            let (left, top) = scrollable_widget.translate_coordinates(drawing_area, hvalue, vvalue).unwrap();

            // TODO: if the page size is greater than the canvas size, we should probably trim to only the area covered by the actual canvas

            // Otherwise, adjust the viewport to the scroll values
            CanvasViewport {
                width:              full_viewport.width,
                height:             full_viewport.height,
                viewport_x:         left,                   // Scaled by translate_coordinates
                viewport_y:         top,                    // Scaled by translate_coordinates
                viewport_width:     hpagesize * scale,
                viewport_height:    vpagesize * scale
            }
        }
    }

    ///
    /// Retrieves the viewport for a canvas
    ///
    fn get_viewport(drawing_area: &gtk::Widget, allocation: &gtk::Allocation) -> CanvasViewport {
        // The scale factor is used to ensure we get a 1:1 pixel ratio for our drawing area
        let scale_factor = drawing_area.get_scale_factor();

        // Search for a scrollable parent to base the viewport upon
        let mut scrollable  = None;
        let mut parent      = Some(drawing_area.clone().upcast::<gtk::Widget>());
        while parent.is_some() && scrollable.is_none() {
            scrollable  = parent.clone().and_then(|parent| parent.dynamic_cast::<gtk::Scrollable>().ok());
            parent      = parent.and_then(|parent| parent.get_parent());
        }

        // Generate a viewport
        let viewport = CanvasViewport {
            width:              allocation.width.max(1) * scale_factor,
            height:             allocation.height.max(1) * scale_factor,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     allocation.width.max(1) * scale_factor,
            viewport_height:    allocation.height.max(1) * scale_factor
        };

        // Clip to the scrollable region if there is one
        match scrollable {
            Some(scrollable)    => Self::clip_viewport_to_scrollable(viewport, &scrollable, drawing_area),
            None                => viewport
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

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Drawing areas can't have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }

    fn draw_manual(&self, context: &cairo::Context) {
        Self::perform_draw(&self.as_widget, context, &self.core);
    }
}
