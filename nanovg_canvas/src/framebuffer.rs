use gl;

///
/// Represents an OpenGL framebuffer
/// 
pub struct FrameBuffer {
    /// The ID of the render buffer used for this frame buffer
    renderbuffer_id: u32,

    /// The ID of the framebuffer
    framebuffer_id: u32
}

impl FrameBuffer {
    ///
    /// Retrieves the currently set framebuffer (for unbinding purposes)
    /// 
    pub fn get_current() -> u32 {
        unsafe {
            let mut draw_framebuffer = 0;
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut draw_framebuffer);
            draw_framebuffer as u32
        }
    }

    ///
    /// Creates a new framebuffer in the current context
    /// 
    pub fn new(width: i32, height: i32) -> FrameBuffer {
        unsafe {
            let active_framebuffer = Self::get_current();

            // Generate a framebuffer
            let mut new_renderbuffer_id = 0;
            gl::GenRenderbuffers(1, &mut new_renderbuffer_id);

            let mut new_framebuffer_id = 0;
            gl::GenFramebuffers(1, &mut new_framebuffer_id);
            
            gl::BindFramebuffer(gl::FRAMEBUFFER, new_framebuffer_id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, new_renderbuffer_id);

            // Render buffer is a MSAA render buffer with the specified width and height
            gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::RGBA8, width, height);

            // Link to the framebuffer
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, new_renderbuffer_id);

            // Check for readiness
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Failed to initialise framebuffer");
            }

            // Reset the framebuffer binding
            gl::BindFramebuffer(gl::FRAMEBUFFER, active_framebuffer);

            // Return the resulting framebuffer
            FrameBuffer {
                renderbuffer_id:    new_renderbuffer_id,
                framebuffer_id:     new_framebuffer_id
            }
        }
    }
}

///
/// Dropping the framebuffer will attempt to deallocate its resources, but may fail if the GL context has changed since it was created
/// 
impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.framebuffer_id);
            gl::DeleteRenderbuffers(1, &self.renderbuffer_id);
        }
    }
}