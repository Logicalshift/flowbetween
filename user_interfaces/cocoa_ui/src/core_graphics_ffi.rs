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

#[repr(C)] pub struct CGMutablePath { _private: [u8; 0] }
pub type CGMutablePathRef = *mut CGMutablePath;

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub struct CGAffineTransform {
    pub a: CGFloat,
    pub b: CGFloat,
    pub c: CGFloat,
    pub d: CGFloat,
    pub tx: CGFloat,
    pub ty: CGFloat
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub struct CGSize {
    pub width: CGFloat,
    pub height: CGFloat
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub enum CGBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,

    Clear,
    Copy,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    XOR,
    PlusDarker,
    PlusLighter
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub enum CGLineJoin {
    Miter,
    Round,
    Bevel
}

#[derive(Copy, Clone, Debug)]
#[repr(C)] pub enum CGLineCap {
    Butt,
    Round,
    Square
}

unsafe impl objc::Encode for CGPoint {
    fn encode() -> objc::Encoding {
        let encoding = format!("{{CGPoint={}{}}}",
                               f64::encode().as_str(),
                               f64::encode().as_str());
        unsafe { objc::Encoding::from_str(&encoding) }
    }
}

unsafe impl objc::Encode for CGSize {
    fn encode() -> objc::Encoding {
        let encoding = format!("{{CGSize={}{}}}",
                               f64::encode().as_str(),
                               f64::encode().as_str());
        unsafe { objc::Encoding::from_str(&encoding) }
    }
}

unsafe impl objc::Encode for CGRect {
    fn encode() -> objc::Encoding {
        let encoding = format!("{{CGRect={}{}}}",
                               CGPoint::encode().as_str(),
                               CGSize::encode().as_str());
        unsafe { objc::Encoding::from_str(&encoding) }
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern {
    pub static kCGColorSpaceSRGB: CFStringRef;
    pub static CGAffineTransformIdentity: CGAffineTransform;

    pub fn CGColorSpaceRetain(colorspace: CGColorSpaceRef);
    pub fn CGColorSpaceRelease(colorspace: CGColorSpaceRef);
    pub fn CGColorSpaceCreateWithName(name: CFStringRef) -> CGColorSpaceRef;

    pub fn CGColorRetain(color: CGColorRef);
    pub fn CGColorRelease(color: CGColorRef);
    pub fn CGColorCreate(colorspace: CGColorSpaceRef, components: *const CGFloat) -> CGColorRef;

    pub fn CGAffineTransformTranslate(t: CGAffineTransform, tx: CGFloat, ty: CGFloat) -> CGAffineTransform;
    pub fn CGAffineTransformScale(t: CGAffineTransform, sx: CGFloat, sy: CGFloat) -> CGAffineTransform;
    pub fn CGAffineTransformRotate(t: CGAffineTransform, angle: CGFloat) -> CGAffineTransform;
    pub fn CGAffineTransformInvert(t: CGAffineTransform) -> CGAffineTransform;
    pub fn CGAffineTransformConcat(t1: CGAffineTransform, t2: CGAffineTransform) -> CGAffineTransform;
    pub fn CGPointApplyAffineTransform(CGPoint: CGPoint, t: CGAffineTransform) -> CGPoint;

    pub fn CGPathCreateMutable() -> CGMutablePathRef;
    pub fn CGPathCreateMutableCopy(path: CGMutablePathRef) -> CGMutablePathRef;
    pub fn CGPathRetain(path: CGMutablePathRef);
    pub fn CGPathRelease(path: CGMutablePathRef);
    pub fn CGPathCloseSubpath(path: CGMutablePathRef);
    pub fn CGPathMoveToPoint(path: CGMutablePathRef, m: *const CGAffineTransform, x: CGFloat, y: CGFloat);
    pub fn CGPathAddLineToPoint(path: CGMutablePathRef, m: *const CGAffineTransform, x: CGFloat, y: CGFloat);
    pub fn CGPathAddCurveToPoint(path: CGMutablePathRef, m: *const CGAffineTransform, cp1x: CGFloat, cp1y: CGFloat, cp2x: CGFloat, cp2y: CGFloat, x: CGFloat, y: CGFloat);

    pub fn CGContextRetain(ctxt: CGContextRef);
    pub fn CGContextRelease(ctxt: CGContextRef);
    pub fn CGContextSaveGState(ctxt: CGContextRef);
    pub fn CGContextRestoreGState(ctxt: CGContextRef);
    pub fn CGContextBeginPath(ctxt: CGContextRef);
    pub fn CGContextMoveToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextAddLineToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextClosePath(ctxt: CGContextRef);
    pub fn CGContextAddCurveToPoint(ctxt: CGContextRef, cp1x: CGFloat, cp1y: CGFloat, cp2x: CGFloat, cp2y: CGFloat, x: CGFloat, y: CGFloat);
    pub fn CGContextFillPath(ctxt: CGContextRef);
    pub fn CGContextStrokePath(ctxt: CGContextRef);
    pub fn CGContextSetLineWidth(ctxt: CGContextRef, width: CGFloat);
    pub fn CGContextSetLineJoin(ctxt: CGContextRef, join: CGLineJoin);
    pub fn CGContextSetLineCap(ctxt: CGContextRef, cap: CGLineCap);
    pub fn CGContextSetFillColorWithColor(ctxt: CGContextRef, color: CGColorRef);
    pub fn CGContextSetStrokeColorWithColor(ctxt: CGContextRef, color: CGColorRef);
    pub fn CGContextConcatCTM(ctxt: CGContextRef, transform: CGAffineTransform);
    pub fn CGContextGetCTM(ctxt: CGContextRef) -> CGAffineTransform;
    pub fn CGContextSetBlendMode(ctxt: CGContextRef, blendMode: CGBlendMode);
    pub fn CGContextAddPath(ctxt: CGContextRef, path: CGMutablePathRef);
    pub fn CGContextClearRect(ctxt: CGContextRef, rect: CGRect);
    pub fn CGContextClip(ctxt: CGContextRef);
}

pub trait CFReleasable {
    fn retain(&self) -> Self;
    fn release(&self);
}

impl CFReleasable for CGContextRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGContextRetain(*self); }
        *self
    }

    #[inline] fn release(&self) {
        unsafe { CGContextRelease(*self); }
    }
}

impl CFReleasable for CGColorRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGColorRetain(*self); }
        *self
    }

    #[inline] fn release(&self) {
        unsafe { CGColorRelease(*self); }
    }
}

impl CFReleasable for CGColorSpaceRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGColorSpaceRetain(*self); }
        *self
    }

    #[inline] fn release(&self) {
        unsafe { CGColorSpaceRelease(*self); }
    }
}

impl CFReleasable for CGMutablePathRef {
    #[inline] fn retain(&self) -> Self {
        unsafe { CGPathRetain(*self); }
        *self
    }

    #[inline] fn release(&self) {
        unsafe { CGPathRelease(*self); }
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

impl<T: CFReleasable> Drop for CFRef<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: CFReleasable> From<T> for CFRef<T> {
    fn from(val: T) -> CFRef<T> { CFRef(val) }
}
