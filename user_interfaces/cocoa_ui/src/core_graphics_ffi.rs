//! FFI for core graphics functions

pub enum CGContext {}
pub type CGContextRef = *mut CGContext;

#[link(name = "CoreGraphics", kind = "framework")]
extern {
    pub fn CGContextBeginPath(ctxt: CGContextRef);
}
