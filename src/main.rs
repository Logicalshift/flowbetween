//!
//! # FlowBetween HTTP server
//!

extern crate ui;
extern crate flo;
extern crate http_ui;
extern crate binding;
extern crate animation;
extern crate anim_sqlite;
extern crate static_files;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate iron;
extern crate mount;
extern crate tokio_core;

mod http_session;

use iron::*;
use mount::*;
use tokio_core::reactor::Core;
use std::sync::*;
use std::thread;

use static_files::*;
use http_ui::*;

use self::http_session::*;

const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVER_ADDR: &str     = "0.0.0.0:3000";
const WS_SERVER_ADDR: &str  = "0.0.0.0:3001";

fn main() {
    // Create the web session structure
    let sessions = Arc::new(WebSessions::new());
    
    // Set up an iron server
    let mut mount   = Mount::new();
    let web_ui      = UiHandler::<FlowBetweenSession>::from_sessions(sessions.clone());
    mount.mount("/", flowbetween_static_files());
    mount.mount("/flowbetween/session", web_ui);
    
    // Run the WS server
    thread::spawn(move || {
        // Set up a websockets server
        let mut tokio_core  = Core::new().unwrap();
        let ws_handle       = Arc::new(tokio_core.handle());
        let ws_handler      = WebSocketHandler::from_sessions(sessions.clone());
        let ws_stream       = ws_handler.create_server(WS_SERVER_ADDR, ws_handle);

        println!("{} v{} preparing to serve websocket requests at {}", PACKAGE_NAME, PACKAGE_VERSION, WS_SERVER_ADDR);
        tokio_core.run(ws_stream).unwrap();
    });

    // Run the iron server
    println!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, SERVER_ADDR);

    Iron::new(mount).http(SERVER_ADDR).unwrap();
}
