use crate::gtk_action::*;
use crate::gtk_thread::*;
use crate::widgets::*;
use crate::widgets::basic_widget::*;

use gtk::prelude::*;

use std::cell::*;
use std::rc::*;

///
/// The GFX canvas widget is a canvas that renders via the GFX library
///
pub struct FloGfxCanvasWidget {
    // The ID of this widget
    id: WidgetId,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget,

    /// The widget as a GL area
    as_glarea: gtk::GLArea
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

        // Set it up
        as_glarea.set_has_alpha(true);
        as_glarea.set_has_stencil_buffer(true);

        // Initialise on realize
        Self::on_realize(&mut as_glarea);
        Self::on_render(&mut as_glarea);

        FloGfxCanvasWidget {
            id,
            as_widget,
            as_glarea
        }
    }

    ///
    /// Installs the callback that deals with realizing the GLArea
    ///
    fn on_realize(glarea: &mut gtk::GLArea) {
        glarea.connect_realize(move |gl_widget| { 
            gl_widget.make_current();
        });
    }

    ///
    /// Installs the callback that deals with rendering the GLArea
    ///
    fn on_render(glarea: &mut gtk::GLArea) {
        glarea.connect_render(move |gl_widget, _ctxt| {
            let allocation      = gl_widget.get_allocation();
            let scale           = gl_widget.get_scale_factor();

            // Prepare to render
            unsafe {
                gl::ClearColor(0.5, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::Viewport(0, 0, allocation.width*scale, allocation.height*scale);
            }

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
