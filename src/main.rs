//!
//! # FlowBetween HTTP server
//!

extern crate ui;
extern crate flo;
extern crate curves;
extern crate canvas;
extern crate http_ui;
extern crate binding;
extern crate animation;
extern crate static_files;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate iron;
extern crate mount;
extern crate desync;

mod http_session;

use iron::*;
use mount::*;

use static_files::*;
use http_ui::*;

use self::http_session::*;

const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVER_ADDR: &str     = "0.0.0.0:3000";

fn main() {
    let mut mount = Mount::new();
    mount.mount("/", flowbetween_static_files());
    mount.mount("/flowbetween/session", UiHandler::<FlowBetweenSession>::new());

    println!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, SERVER_ADDR);

    Iron::new(mount).http(SERVER_ADDR).unwrap();
}
