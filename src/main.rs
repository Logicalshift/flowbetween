//!
//! # FlowBetween HTTP server
//!

extern crate iron;
extern crate mount;
extern crate static_files;
extern crate http_ui;

use iron::*;
use mount::*;

use static_files::*;
use http_ui::*;

const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVER_ADDR: &str     = "127.0.0.1:3000";

fn main() {
    let mut mount = Mount::new();
    mount.mount("/", flowbetween_static_files());
    mount.mount("/flowbetween/session", UiHandler::<NullSession>::new());

    println!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, SERVER_ADDR);

    Iron::new(mount).http(SERVER_ADDR).unwrap();
}
