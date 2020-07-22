use gl;

use std::ptr;
use std::ops::{Deref};

///
/// Abstraction that manages an OpenGL texture
///
pub struct Texture {
    texture: gl::types::GLuint
}

impl Texture {
    ///
    /// Creates a new OpenGL texture object
    ///
    pub fn new() -> Texture {
        unsafe {
            let mut new_texture = 0;
            gl::GenTextures(1, &mut new_texture);

            Texture {
                texture: new_texture
            }
        }
    }

    ///
    /// Associates an empty image with this texture
    ///
    pub fn create_empty(&mut self, width: u16, height: u16) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TextureParameteri(self.texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(self.texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as gl::types::GLsizei, height as gl::types::GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
        }
    }

    ///
    /// Creates an empty MSAA texture
    ///
    pub fn create_empty_msaa(&mut self, width: u16, height: u16, samples: usize) {
        unsafe {
            // Clamp the number of samples to the maximum supported by the driver
            let mut max_samples = 1;
            gl::GetIntegerv(gl::MAX_COLOR_TEXTURE_SAMPLES, &mut max_samples);
            let samples = max_samples.min(samples as i32);

            // Set up a MSAA texture
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.texture);

            gl::TextureParameteri(self.texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(self.texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE, samples, gl::RGBA, width as gl::types::GLsizei, height as gl::types::GLsizei, gl::FALSE);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.texture);
        }
    }
}

impl Deref for Texture {
    type Target = gl::types::GLuint;

    fn deref(&self) -> &gl::types::GLuint {
        &self.texture
    }
}
