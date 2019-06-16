#![warn(bare_trait_objects)]

extern crate flo_ui;
extern crate flo_canvas;
extern crate flo_http_ui;
extern crate flo_logging;
extern crate flo_static_files;

extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
extern crate futures;
extern crate bytes;
#[macro_use] extern crate lazy_static;
extern crate serde_json;
extern crate percent_encoding;

mod actix_session;
mod session_handler;
mod session_websocket_handler;
mod session_resource_handler;
mod static_file_handler;

pub use self::actix_session::*;
pub use self::session_handler::*;
pub use self::session_websocket_handler::*;
pub use self::session_resource_handler::*;
pub use self::static_file_handler::*;
