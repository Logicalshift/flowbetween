//! FFI for core graphics functions

#[cfg(target_pointer_width = "32")] use std::os::raw::c_float;
#[cfg(target_pointer_width = "64")] use std::os::raw::c_double;
use std::ops::Deref;

#[cfg(target_pointer_width = "64")] pub type CGFloat = c_double;
#[cfg(target_pointer_width = "32")] pub type CGFloat = c_float;

#[repr(C)] pub struct __CFString { _private: [u8; 0] }
pub type CFStringRef = *mut __CFString;

#[repr(C)] pub struct CGContext {  _private: [u8; 0] }
pub type CGContextRef = *mut CGContext;

#[repr(C)] pub struct CGColorSpace { _private: [u8; 0] }
pub type CGColorSpaceRef = *mut CGColorSpace;

#[repr(C)] pub struct CGColor { _private: [u8; 0] }
pub type CGColorRef = *mut CGColor;

#[link(name = "CoreGraphics", kind = "framework")]
extern {
    pub static kCGColorSpaceSRGB: CFStringRef;

    pub fn CGColorSpaceRetain(colorspace: CGColorSpaceRef);
    pub fn CGColorSpaceRelease(colorspace: CGColorSpaceRef);
    pub fn CGColorSpaceCreateWithName(name: CFStringRef) -> CGColorSpaceRef;

    pub fn CGColorRetain(color: CGColorRef);
    pub fn CGColorRelease(color: CGColorRef);
    pub fn CGColorCreate(colorspace: CGColorSpaceRef, components: *const CGFloat) -> CGColorRef;

    pub fn CGContextRetain(ctxt: CGContextRef);
    pub fn CGContextRelease(ctxt: CGContextRef);
    pub fn CGContextBeginPath(ctxt: CGContextRef);
    pub fn CGContextMoveToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextAddLineToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextClosePath(ctxt: CGContextRef);
    pub fn CGContextAddCurveToPoint(ctxt: CGContextRef, cp1x: CGFloat, cp1y: CGFloat, cp2x: CGFloat, cp2y: CGFloat, x: CGFloat, y: CGFloat);
    pub fn CGContextFillPath(ctxt: CGContextRef);
    pub fn CGContextStrokePath(ctxt: CGContextRef);
    pub fn CGContextSetLineWidth(ctxt: CGContextRef, width: CGFloat);
    pub fn CGContextSetFillColorWithColor(ctxt: CGContextRef, color: CGColorRef);
    pub fn CGContextSetStrokeColorWithColor(ctxt: CGContextRef, color: CGColorRef);
}

pub trait CFReleasable { 
    fn retain(&self) -> Self;
    fn release(self); 
}

impl CFReleasable for CGContextRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGContextRetain(*self); }
        *self
    }

    #[inline] fn release(self) {
        unsafe { CGContextRelease(self); }
    }
}

impl CFReleasable for CGColorRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGColorRetain(*self); }
        *self
    }

    #[inline] fn release(self) {
        unsafe { CGColorRelease(self); }
    }
}

impl CFReleasable for CGColorSpaceRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGColorSpaceRetain(*self); }
        *self
    }

    #[inline] fn release(self) {
        unsafe { CGColorSpaceRelease(self); }
    }
}

pub struct CFRef<T: CFReleasable>(T);

impl<T: CFReleasable> Clone for CFRef<T> {
    #[inline] fn clone(&self) -> CFRef<T> {
        CFRef(self.0.retain())
    }
}

impl<T: CFReleasable> Deref for CFRef<T> {
    type Target = T;

    #[inline] fn deref(&self) -> &T { &self.0 }
}

impl<T: CFReleasable> From<T> for CFRef<T> {
    fn from(val: T) -> CFRef<T> { CFRef(val) }
}
