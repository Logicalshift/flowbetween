#![warn(bare_trait_objects)]

extern crate flo_ui as ui;
extern crate flo_curves as curves;
extern crate flo_canvas as canvas;
extern crate flo_binding as binding;
extern crate flo_animation as animation;

extern crate desync;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate itertools;

pub mod editor;
pub mod style;
pub mod animation_canvas;
pub mod tools;
pub mod standard_tools;
pub mod menu;
pub mod color;

mod model;

pub use self::editor::*;
