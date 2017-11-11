//!
//! An abstract representation of a vector canvas object 
//!

extern crate futures;

mod draw;
mod color;
mod canvas;
mod encoding;
mod transform2d;

pub use self::draw::*;
pub use self::color::*;
pub use self::canvas::*;
pub use self::encoding::*;
pub use self::transform2d::*;
