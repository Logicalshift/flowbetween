#![warn(bare_trait_objects)]

extern crate flo_ui;
extern crate flo_curves;
extern crate flo_stream;
extern crate flo_canvas;
extern crate flo_binding;
extern crate flo_ui_files;
extern crate flo_animation;

extern crate desync;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate itertools;

pub mod chooser;
pub mod editor;
pub mod style;
pub mod animation_canvas;
pub mod tools;
pub mod standard_tools;
pub mod menu;
pub mod color;

mod model;

pub use self::editor::*;
