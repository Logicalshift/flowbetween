use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;
use gl;
use nanovg;

use std::rc::*;
use std::cell::*;

///
/// NanoVG core data, shared with event handlers
/// 
struct NanoVgCore {
    /// The context, if it exists
    context: Option<nanovg::Context>
}

///
/// Uses NanoVG to draw using OpenGL on a widget
/// 
pub struct FloNanoVgWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The GTK GLArea widget (needs to be explicitly retained to avoid random self-destruction)
    _gl_widget: gtk::GLArea,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget
}

impl FloNanoVgWidget {
    ///
    /// Creates a new NanoVG widget with a particular GL area as the target
    /// 
    pub fn new<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloNanoVgWidget {
        // Fetch the GL widget and its widget representation
        let gl_widget = widget.upcast::<gtk::GLArea>();
        let as_widget = gl_widget.clone().upcast::<gtk::Widget>();

        // Create the core data
        let core = NanoVgCore {
            context: None
        };
        let core = Rc::new(RefCell::new(core));

        // Configure the GL area
        gl_widget.set_has_alpha(true);
        gl_widget.set_has_stencil_buffer(true);

        // Simple realize event
        {
            let core = Rc::clone(&core);
            gl_widget.connect_realize(move |gl_widget| {
                let mut core = core.borrow_mut();

                // Set the context
                gl_widget.make_current();

                // Create the nanovg context
                let context     = nanovg::ContextBuilder::new()
                    .stencil_strokes()
                    .antialias()
                    .build()
                    .expect("Failed to create NanoVG context");
                core.context    = Some(context);
            });
        }

        // Simple rendering to test out our widget
        {
            let core = Rc::clone(&core);
            gl_widget.connect_render(move |gl_widget, _ctxt| { 
                let core        = core.borrow();
                let allocation  = gl_widget.get_allocation();
                let context     = core.context.as_ref().unwrap();
                let scale       = gl_widget.get_scale_factor();

                // Prepare to render
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Viewport(0, 0, allocation.width*scale, allocation.height*scale);
                }

                context.frame((allocation.width, allocation.height), scale as f32, |frame| {
                    frame.path(|path| {
                        path.rect((100.0, 100.0), (1980.0-200.0, 1080.0-200.0));
                        path.fill(nanovg::Color::new(0.5, 0.5, 0.8, 0.5), Default::default());
                    }, nanovg::PathOptions { clip: nanovg::Clip::None, composite_operation: nanovg::CompositeOperation::Basic(nanovg::BasicCompositeOperation::SourceOver), alpha: 1.0, transform: None });

                    frame.path(|path| {
                        path.circle((1980.0/2.0, 1080.0/2.0), 100.0);
                        path.fill(nanovg::Color::new(0.8, 0.5, 0.2, 1.0), Default::default());
                    }, nanovg::PathOptions { clip: nanovg::Clip::None, composite_operation: nanovg::CompositeOperation::Basic(nanovg::BasicCompositeOperation::SourceOver), alpha: 1.0, transform: None });
                });

                Inhibit(true)
            });
        }

        // Generate the result
        FloNanoVgWidget {
            id:         widget_id,
            _gl_widget: gl_widget,
            as_widget:  as_widget
        }
    }
}

impl GtkUiWidget for FloNanoVgWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // NanoVG widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}