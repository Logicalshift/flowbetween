#![warn(bare_trait_objects)]

extern crate flo_ui as ui;
extern crate flo_canvas as canvas;
extern crate flo_binding as binding;
extern crate flo_logging;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate uuid;
extern crate percent_encoding;
extern crate itertools;
extern crate futures;
extern crate desync;

extern crate bytes;

mod http_user_interface;
mod http_controller;
mod http_session;
mod sessions;
mod update;
mod event;
mod htmlcontrol;
mod ui_handler;
mod null_session;
pub mod minidom;
mod canvas_update;
mod lazy_future;

pub use self::http_user_interface::*;
pub use self::http_controller::*;
pub use self::http_session::*;
pub use self::sessions::*;
pub use self::update::*;
pub use self::event::*;
pub use self::htmlcontrol::*;
pub use self::ui_handler::*;
pub use self::null_session::*;
pub use self::canvas_update::*;
