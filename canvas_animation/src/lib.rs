//!
//! `flo_canvas_animation` is a crate that allows annotating a static vector image with
//! animations. Unlike similar libraries, this library is designed around the concept of
//! 'regions' and can automatically divide up the drawing that's supplied to it.
//!
//! As well as animations, this crate can be used to apply effects to regions (eg: drop shadows)
//!

mod animation_layer;

pub use animation_layer::*;
