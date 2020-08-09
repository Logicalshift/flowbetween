use crate::gtk_action::*;
use crate::gtk_thread::*;
use crate::widgets::*;
use crate::widgets::flo_layout::*;
use crate::widgets::basic_widget::*;
use crate::widgets::layout_settings::*;

use flo_canvas::*;
use flo_render;
use flo_render::{Vertex2D, VertexBufferId, Rgba8, RenderAction, Matrix};
use flo_render_canvas::*;
use gtk::prelude::*;
use cairo;

use futures::prelude::*;
use futures::executor;

use std::cell::*;
use std::rc::*;
use std::mem;
use std::time;

///
/// Mutable data used by different parts of the hardware rendering widghet
///
struct FloRenderWidgetCore {
    /// The renderer for this widget
    renderer: Option<flo_render::GlRenderer>,

    /// The canvas renderer turns canvas instructions into renderer instructions
    canvas_renderer: CanvasRenderer,

    /// Any canvas operations that are waiting to be sent to the renderer
    waiting_to_render: Vec<Draw>,

    /// The scale applied to the widget
    scale: f32,

    /// The data attached to the widget
    widget_data: Rc<WidgetData>,

    /// The ID of the widget that this core belongs to
    widget_id: WidgetId,
}

///
/// The render canvas widget is a canvas that renders via the flo_render library
///
pub struct FloRenderCanvasWidget {
    // The ID of this widget
    id: WidgetId,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget,

    /// The widget as a GL area
    as_glarea: gtk::GLArea,

    /// Shared data used by the widget callbacks
    core: Rc<RefCell<FloRenderWidgetCore>>
}

impl FloRenderCanvasWidget {
    ///
    /// Creates a new hardware rendering canvas widget that renders to the specified GL area
    ///
    pub fn new_opengl<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W, data: Rc<WidgetData>) -> FloRenderCanvasWidget {
        // Get the widget as a GL area
        let id              = widget_id;
        let mut as_glarea   = widget.clone().upcast::<gtk::GLArea>();
        let as_widget       = as_glarea.clone().upcast::<gtk::Widget>();
        let core            = Rc::new(RefCell::new(FloRenderWidgetCore::new(widget_id, Rc::clone(&data))));

        // This needs to be clipped to the viewport by whatever lays it out
        let layout_settings = LayoutSettings { clip_to_viewport: true };
        data.set_widget_data(widget_id, layout_settings);
        debug_assert!(data.get_widget_data::<LayoutSettings>(widget_id).is_some());

        // Set it up
        as_glarea.set_has_alpha(true);
        as_glarea.set_has_stencil_buffer(true);

        // Initialise on realize
        Self::on_realize(&mut as_glarea, Rc::clone(&core));
        Self::on_render(&mut as_glarea, Rc::clone(&core));

        FloRenderCanvasWidget {
            id:         id,
            as_widget:  as_widget,
            as_glarea:  as_glarea,
            core:       core
        }
    }

    ///
    /// Installs the callback that deals with realizing the GLArea
    ///
    fn on_realize(glarea: &mut gtk::GLArea, core: Rc<RefCell<FloRenderWidgetCore>>) {
        glarea.connect_realize(move |gl_widget| { 
            // Borrow the core
            let mut core = core.borrow_mut();

            // Get the window dimensions
            let allocation      = gl_widget.get_allocation();
            let scale           = gl_widget.get_scale_factor();
            let width           = allocation.width * scale;
            let height          = allocation.height * scale;

            // Make the context the current context
            gl_widget.make_current();

            // Set up the renderer
            core.renderer = Some(flo_render::GlRenderer::new());
        });
    }

    ///
    /// Installs the callback that deals with rendering the GLArea
    ///
    fn on_render(glarea: &mut gtk::GLArea, core: Rc<RefCell<FloRenderWidgetCore>>) {
        glarea.connect_render(move |gl_widget, _ctxt| {
            let start       = time::SystemTime::now();

            // Borrow the core to use while rendering
            let mut core    = core.borrow_mut();

            executor::block_on(async move {
                // Borrowing trick here (DerefMut is not quite transparent and we need mutable references to multiple fields)
                let core                = &mut *core;

                // Get the current size of the control
                let position            = core.widget_data.get_widget_data::<WidgetPosition>(core.widget_id);
                let position            = match position { None => { return; }, Some(position) => position };
                let position            = position.borrow();

                let viewport            = core.widget_data.get_widget_data::<ViewportPosition>(core.widget_id);
                let viewport            = match viewport { None => { return; }, Some(viewport) => viewport };
                let viewport            = viewport.borrow();

                let allocation          = gl_widget.get_allocation();
                let scale               = gl_widget.get_scale_factor();

                // Set whatever is set as the current framebuffer as the render target
                let viewport_x          = viewport.x1 as f32;
                let viewport_y          = (position.y2 - viewport.y1 - allocation.height as f64) as f32;
                let viewport_width      = allocation.width as f32;
                let viewport_height     = allocation.height as f32;

                // Set up the canvas renderer
                let canvas_renderer     = &mut core.canvas_renderer;
                let waiting_to_render   = &mut core.waiting_to_render;
                let renderer            = &mut core.renderer;

                let window_width        = (position.x2-position.x1) as f32;
                let window_height       = (position.y2-position.y1) as f32;

                // Multiply everything by the scale to get native resolution
                let scale               = scale as f32;
                core.scale              = scale;

                let viewport_x          = viewport_x * scale;
                let viewport_y          = viewport_y * scale;
                let viewport_width      = viewport_width * scale;
                let viewport_height     = viewport_height * scale;
                let window_width        = window_width * scale;
                let window_height       = window_height * scale;

                canvas_renderer.set_viewport(viewport_x..(viewport_x+viewport_width), viewport_y..(viewport_y+viewport_height), window_width, window_height, scale);

                if let Some(renderer) = renderer {
                    // Set up the renderer to render to the current framebuffer
                    renderer.prepare_to_render_to_active_framebuffer(viewport_width as usize, viewport_height as usize);

                    // Draw any pending instructions
                    let mut pending_drawing = vec![];
                    mem::swap(&mut pending_drawing, waiting_to_render);
                    let render_stream       = canvas_renderer.draw(pending_drawing.into_iter());

                    // Perform the rendering
                    let render_actions      = render_stream.collect::<Vec<_>>().await;

                    renderer.render(render_actions);

                    // Finish up
                    renderer.flush();

                    // Update the coordinates transform in the widget data
                    core.update_widget_transform(allocation.height as f32);
                }
            });

            let render_time = time::SystemTime::now().duration_since(start).unwrap();
            if render_time.as_micros() > 16000 {
                println!("Rendering took {} microseconds", render_time.as_micros());
            }

            Inhibit(true)
        });
    }
}

impl GtkUiWidget for FloRenderCanvasWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            &GtkWidgetAction::Content(WidgetContent::Draw(ref drawing)) => { 
                let mut core = self.core.borrow_mut();

                // Clear the entire list of things to render if there's a ClearCanvas anywhere in the drawing
                for draw in drawing.iter() {
                    if let Draw::ClearCanvas = draw {
                        core.waiting_to_render = vec![];
                    }
                }

                // Add to the list to render next time this control is updated
                core.waiting_to_render.extend(drawing.into_iter().cloned());

                // Mark the widget as needing a render
                self.as_glarea.queue_render();
            },

            other_action                                                => process_basic_widget_action(self, flo_gtk, other_action)
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Canvas widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

impl FloRenderWidgetCore {
    ///
    /// Creates a new render widget core
    ///
    pub fn new(widget_id: WidgetId, data: Rc<WidgetData>) -> FloRenderWidgetCore {
        let default_render = vec![Draw::ClearCanvas];

        FloRenderWidgetCore {
            renderer:           None,
            canvas_renderer:    CanvasRenderer::new(),
            waiting_to_render:  default_render,
            scale:              1.0,
            widget_data:        data,
            widget_id:          widget_id
        }
    }

    ///
    /// Updates the widget transform from the active transform for the renderer
    ///
    fn update_widget_transform(&self, height: f32) {
        let active_transform            = self.canvas_renderer.get_active_transform();
        let (viewport_x, viewport_y)    = self.canvas_renderer.get_viewport();

        // GTK uses flipped coordinates
        let flip_window                 = Transform2D::scale(1.0, -1.0) * Transform2D::translate(0.0, -height);

        // The coordinates are relative to the GL area, which has a fixed viewport
        let add_viewport                = Transform2D::translate(viewport_x.start/self.scale, -viewport_y.start/self.scale);

        // Invert to get the transformation from canvas coordinates to window coordinates
        let active_transform            = (flip_window*add_viewport*active_transform).invert().unwrap();

        // Flip the inverted transform and convert the matrix to the format used by Gtk
        let Transform2D([a, b, _c])     = active_transform;
        let cairo_matrix                = cairo::Matrix::new(a[0] as f64, b[0] as f64, a[1] as f64, b[1] as f64, a[2] as f64, b[2] as f64);

        self.widget_data.set_widget_data(self.widget_id, cairo_matrix);
    }
}
