use super::error::*;
use super::opengl::*;
use super::offscreen_trait::*;

use flo_render_gl_offscreen::wgl;
use flo_render_gl_offscreen::winapi;
use flo_render_gl_offscreen::winapi::um::winuser::{CreateWindowExW, RegisterClassExW, GetDC, WNDCLASSEXW, CS_OWNDC, WS_OVERLAPPEDWINDOW};
use flo_render_gl_offscreen::winapi::um::wingdi::{PIXELFORMATDESCRIPTOR, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_DOUBLEBUFFER, PFD_TYPE_RGBA, PFD_MAIN_PLANE};
use flo_render_gl_offscreen::winapi::um::libloaderapi::{GetModuleHandleW};

use std::mem;
use std::ptr;
use std::ffi::{OsStr};
use std::os::windows::ffi::{OsStrExt};

struct WglOffscreenRenderContext {

}

///
/// Performs on-startup initialisation steps for offscreen rendering
///
/// Only required if not using a toolkit renderer (eg, in an HTTP renderer or command-line tool). Will likely replace
/// the bindings for any GUI toolkit, so this is not appropriate for desktop-type apps.
///
/// This version is the WGL version for Windows
///
pub fn initialize_offscreen_rendering() -> Result<impl OffscreenRenderContext, RenderInitError> {
    // See also https://www.khronos.org/opengl/wiki/Creating_an_OpenGL_Context_(WGL)
    unsafe {
        // Set up the window class
        let class_name              = OsStr::new("flo_draw OpenGL window").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let window_name             = OsStr::new("flo_draw OpenGL window").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let mut window_class        = mem::zeroed::<WNDCLASSEXW>();
        window_class.lpszClassName  = class_name.as_ptr();
        window_class.style          = CS_OWNDC;

        if RegisterClassExW(&window_class) != 0 {
            panic!("Unable to register window class for offscreen rendering");
        }

        // Create a window (we never show this or render to it, but we need a device context)
        let window                  = CreateWindowExW(WS_OVERLAPPEDWINDOW, class_name.as_ptr(), window_name.as_ptr(), WS_OVERLAPPEDWINDOW, 0, 0, 1024, 768, ptr::null_mut(), ptr::null_mut(), GetModuleHandleW(ptr::null()), ptr::null_mut());
        if window.is_null() {
            panic!("Unable to create window for offscreen rendering");
        }

        // Fetch the device context for the window
        let dc                      = GetDC(window);
        if dc.is_null() {
            panic!("Unable to fetch device context for offscreen window");
        }

        // Set up the pixel format descriptor
        let pixel_format = PIXELFORMATDESCRIPTOR {
            nSize:              mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion:           1,
            dwFlags:            PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType:         PFD_TYPE_RGBA,
            cColorBits:         32,
            cRedBits:           0,
            cRedShift:          0,
            cGreenBits:         0,
            cGreenShift:        0,
            cBlueBits:          0,
            cBlueShift:         0,
            cAlphaBits:         0,
            cAlphaShift:        0,
            cAccumBits:         0,
            cAccumRedBits:      0,
            cAccumGreenBits:    0,
            cAccumBlueBits:     0,
            cAccumAlphaBits:    0,
            cDepthBits:         24,
            cStencilBits:       8,
            cAuxBuffers:        0,
            iLayerType:         PFD_MAIN_PLANE,
            bReserved:          0,
            dwLayerMask:        0,
            dwVisibleMask:      0,
            dwDamageMask:       0
        };

        // The result is a new WglOffscreenRenderContext
        Ok(WglOffscreenRenderContext {

        })
    }
}

impl OffscreenRenderContext for WglOffscreenRenderContext {
    type RenderTarget = OpenGlOffscreenRenderer;

    ///
    /// Creates a new render target for this context
    ///
    fn create_render_target(&mut self, width: usize, height: usize) -> Self::RenderTarget {
        panic!("Cannot create WGL render target yet");

        unsafe {
            OpenGlOffscreenRenderer::new(width, height)
        }
    }
}
