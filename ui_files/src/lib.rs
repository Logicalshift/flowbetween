#![warn(bare_trait_objects)]

extern crate flo_ui;
extern crate flo_binding;

extern crate dirs;
extern crate uuid;
extern crate desync;
extern crate rusqlite;
#[macro_use] extern crate lazy_static;

mod file_model;
mod open_file_store;
mod file_manager;
pub mod ui;
pub mod sqlite;

pub use self::file_model::*;
pub use self::open_file_store::*;
pub use self::file_manager::*;
