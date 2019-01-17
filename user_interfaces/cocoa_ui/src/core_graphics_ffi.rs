//! FFI for core graphics functions

#[cfg(target_pointer_width = "32")]
use std::os::raw::c_float;

#[cfg(target_pointer_width = "64")]
use std::os::raw::c_double;

#[cfg(target_pointer_width = "64")]
pub type CGFloat = c_double;

#[cfg(target_pointer_width = "32")]
pub type CGFloat = c_float;

pub enum CGContext {}
pub type CGContextRef = *mut CGContext;

#[link(name = "CoreGraphics", kind = "framework")]
extern {
    pub fn CGContextBeginPath(ctxt: CGContextRef);
    pub fn CGContextMoveToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextAddLineToPoint(ctxt: CGContextRef, x: CGFloat, y: CGFloat);
    pub fn CGContextClosePath(ctxt: CGContextRef);
    pub fn CGContextAddCurveToPoint(ctxt: CGContextRef, cp1x: CGFloat, cp1y: CGFloat, cp2x: CGFloat, cp2y: CGFloat, x: CGFloat, y: CGFloat);
    pub fn CGContextFillPath(ctxt: CGContextRef);
    pub fn CGContextStrokePath(ctxt: CGContextRef);
    pub fn CGContextSetLineWidth(ctxt: CGContextRef, width: CGFloat);
}
