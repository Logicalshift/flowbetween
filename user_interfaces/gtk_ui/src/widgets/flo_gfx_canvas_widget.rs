use crate::gtk_action::*;
use crate::gtk_thread::*;
use crate::widgets::*;
use crate::widgets::basic_widget::*;

use flo_gfx::*;
use gtk::prelude::*;

use std::cell::*;
use std::rc::*;

///
/// Mutable data used by different parts of the GFX widghet
///
struct FloGfxWidgetCore {
    /// The renderer for this widget
    renderer: Option<flo_gfx::GlRenderer>
}

///
/// The GFX canvas widget is a canvas that renders via the GFX library
///
pub struct FloGfxCanvasWidget {
    // The ID of this widget
    id: WidgetId,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget,

    /// The widget as a GL area
    as_glarea: gtk::GLArea,

    /// Shared data used by the widget callbacks
    core: Rc<RefCell<FloGfxWidgetCore>>
}

impl FloGfxCanvasWidget {
    ///
    /// Creates a new GFX canvas widget that renders to the specified GL area
    ///
    pub fn new_opengl<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloGfxCanvasWidget {
        // Get the widget as a GL area
        let id              = widget_id;
        let mut as_glarea   = widget.clone().upcast::<gtk::GLArea>();
        let as_widget       = as_glarea.clone().upcast::<gtk::Widget>();
        let core            = Rc::new(RefCell::new(FloGfxWidgetCore::new()));

        // Set it up
        as_glarea.set_has_alpha(true);
        as_glarea.set_has_stencil_buffer(true);

        // Initialise on realize
        Self::on_realize(&mut as_glarea, Rc::clone(&core));
        Self::on_render(&mut as_glarea, Rc::clone(&core));

        FloGfxCanvasWidget {
            id:         id,
            as_widget:  as_widget,
            as_glarea:  as_glarea,
            core:       core
        }
    }

    ///
    /// Installs the callback that deals with realizing the GLArea
    ///
    fn on_realize(glarea: &mut gtk::GLArea, core: Rc<RefCell<FloGfxWidgetCore>>) {
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
            core.renderer = Some(flo_gfx::GlRenderer::new());
        });
    }

    ///
    /// Installs the callback that deals with rendering the GLArea
    ///
    fn on_render(glarea: &mut gtk::GLArea, core: Rc<RefCell<FloGfxWidgetCore>>) {
        glarea.connect_render(move |gl_widget, _ctxt| {
            // Borrow the core
            let mut core = core.borrow_mut();

            // Get the current size of the control
            let allocation      = gl_widget.get_allocation();
            let scale           = gl_widget.get_scale_factor();

            // Set whatever is set as the current framebuffer as the render target
            let width           = allocation.width * scale;
            let height          = allocation.height * scale;

            // Clear the view
            core.renderer.as_mut().map(|renderer| {
                // Set up the renderer to render to the current framebuffer
                renderer.prepare_to_render_to_active_framebuffer(width as usize, height as usize);

                // Perform the rendering
                renderer.render(vec![
                    GfxAction::Clear(Rgba8([0, 0, 0, 0])),
                    GfxAction::CreateVertex2DBuffer(VertexBufferId(1), vec![
                        Vertex2D { pos: [0.0, 1.0],     tex_coord: [0.0, 0.0], color: [255, 0, 0, 255] },
                        Vertex2D { pos: [-1.0, -1.0],   tex_coord: [0.0, 0.0], color: [0, 255, 0, 0] },
                        Vertex2D { pos: [1.0, -1.0],    tex_coord: [0.0, 0.0], color: [0, 0, 255, 128] }
                    ]),
                    GfxAction::DrawTriangles(VertexBufferId(1), 0..3)
                ]);
                renderer.flush();
            });

            Inhibit(true)
        });
    }
}

impl GtkUiWidget for FloGfxCanvasWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            &GtkWidgetAction::Content(WidgetContent::Draw(ref drawing)) => { },
            other_action                                                => process_basic_widget_action(self, flo_gtk, other_action)
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // GFX widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

impl FloGfxWidgetCore {
    ///
    /// Creates a new GFX widget core
    ///
    pub fn new() -> FloGfxWidgetCore {
        FloGfxWidgetCore {
            renderer: None
        }
    }
}
