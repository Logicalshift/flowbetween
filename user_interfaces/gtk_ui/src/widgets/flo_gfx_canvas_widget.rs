use crate::gtk_action::*;
use crate::gtk_thread::*;
use crate::widgets::*;
use crate::widgets::basic_widget::*;

use flo_gfx::*;

use gl;
use gfx_core::format::{Formatted};
use gfx_core::memory::{Typed};
use gtk::prelude::*;
use gfx_device_gl;
use epoxy;

use std::cell::*;
use std::rc::*;

///
/// Mutable data used by different parts of the GFX widghet
///
struct FloGfxWidgetCore {
    /// The renderer for this widget
    renderer: Option<flo_gfx::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory>>
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
            let dimensions      = (width as u16, height as u16, 0u16, gfx_core::texture::AaMode::Single);

            // Make the context the current context
            gl_widget.make_current();

            // Create a new GFX GL device (using epoxy to look up the functions)
            let (device, mut factory)       = gfx_device_gl::create(|s| epoxy::get_proc_addr(s));
            let command_buffer              = factory.create_command_buffer();
            let encoder                     = gfx::Encoder::from(command_buffer);

            // Set up the renderer
            core.renderer = Some(flo_gfx::Renderer::new(device, factory, encoder));

            // Create a render target view that renders to the main frame buffer
            core.use_default_render_target_as_main(dimensions);
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
            let dimensions      = (width as u16, height as u16, 0u16, gfx_core::texture::AaMode::Single);

            core.use_current_framebuffer_as_main_render_target(dimensions);

            // Clear the view
            core.renderer.as_mut().map(|renderer| {
                renderer.render(vec![GfxAction::Clear(Rgba8([0, 0, 128, 255]))]);
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

    ///
    /// Sets the render target to be the default render target for the OpenGL context
    ///
    pub fn use_default_render_target_as_main(&mut self, dimensions: gfx::texture::Dimensions) {
        // Create render targets from the main target
        let color_format                = gfx::format::Rgba8::get_format();
        let stencil_format              = gfx::format::DepthStencil::get_format();
        let (raw_render, raw_stencil)   = gfx_device_gl::create_main_targets_raw(dimensions, color_format.0, stencil_format.0);

        let render_target               = Typed::new(raw_render);
        let stencil                     = Typed::new(raw_stencil);

        self.renderer.as_mut().map(|renderer| renderer.set_main_render_target(render_target, stencil));
    }

    ///
    /// Sets the current framebuffer as the default render target for the OpenGL context
    ///
    pub fn use_current_framebuffer_as_main_render_target(&mut self, dimensions: gfx::texture::Dimensions) {
        // The 'main' framebuffer is not the one GTK wants to use: we need to create a framebuffer from whatever is set
        // We can load in the frame buffer resources by querying OpenGL state and then use the same method that the
        // GFX OpenGL driver uses in `create_main_targets_raw` to attach the renderer.

        // See https://www.khronos.org/opengl/wiki/GLAPI/glGetFramebufferAttachmentParameter to read things like the texture
        // See GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut framebuffer_name); to get the framebuffer ID

        use gfx_core::handle;
        use gfx_core::handle::{Producer};
        use gfx_core::memory::{Bind, Usage};

        // Assume a standard 8BPP framebuffer
        let color_format                = gfx::format::Rgba8::get_format();
        let stencil_format              = gfx::format::DepthStencil::get_format();

        // Create a temporary handle manager
        let mut handle_manager          = handle::Manager::new();

        // Read the current framebuffer information
        let mut framebuffer_id          = 0;
        let mut framebuffer_texture_id  = 0;
        let mut framebuffer_stencil_id  = 0;
        unsafe {
            // Get the current framebuffer ID
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut framebuffer_id);

            // Get the framebuffer texture
            gl::GetFramebufferAttachmentParameteriv(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME, &mut framebuffer_texture_id);
            gl::GetFramebufferAttachmentParameteriv(gl::DRAW_FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME, &mut framebuffer_stencil_id);
        }

        // Convert the IDs to raw textures
        let _framebuffer_id         = framebuffer_id as u32;
        let framebuffer_texture_id  = framebuffer_texture_id as u32;
        let framebuffer_stencil_id  = framebuffer_stencil_id as u32;

        let framebuffer_texture     = handle_manager.make_texture(
            gfx_device_gl::NewTexture::Texture(framebuffer_texture_id),
            gfx::texture::Info {
                levels: 1,
                kind:   gfx::texture::Kind::D2(dimensions.0, dimensions.1, dimensions.3),
                format: color_format.0,
                bind:   Bind::RENDER_TARGET | Bind::TRANSFER_SRC,
                usage:  Usage::Data,
            },
        );

        let stencil_texture         = handle_manager.make_texture(
            gfx_device_gl::NewTexture::Texture(framebuffer_stencil_id),
            gfx::texture::Info {
                levels: 1,
                kind:   gfx::texture::Kind::D2(dimensions.0, dimensions.1, dimensions.3),
                format: stencil_format.0,
                bind:   Bind::DEPTH_STENCIL | Bind::TRANSFER_SRC,
                usage:  Usage::Data,
            },
        );

        // See `create_main_targets_raw` in gfx_device_gl for how this works
        let raw_color       = handle_manager.make_rtv(gfx_device_gl::TargetView::Texture(framebuffer_texture_id, 0), &framebuffer_texture, dimensions);
        let raw_stencil     = handle_manager.make_dsv(gfx_device_gl::TargetView::Texture(framebuffer_stencil_id, 0), &stencil_texture, dimensions);

        // Convert from the raw type
        let render_target   = Typed::new(raw_color);
        let stencil         = Typed::new(raw_stencil);

        // Set in the renderer
        self.renderer.as_mut().map(|renderer| renderer.set_main_render_target(render_target, stencil));
    }
}
