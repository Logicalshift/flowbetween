//!
//! # FlowBetween HTTP server
//!

extern crate flo_ui as ui;
extern crate flo;
extern crate flo_http_ui as http_ui;
extern crate flo_binding as binding;
extern crate flo_animation as animation;
extern crate flo_anim_sqlite as anim_sqlite;
extern crate flo_static_files as static_files;

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
const SERVER_PORT: u32      = 3000;
const WS_SERVER_PORT: u32   = 3001;
const BIND_ADDRESS: &str    = "0.0.0.0";

fn main() {
    // Create the web session structure
    let sessions = Arc::new(WebSessions::new());
    
    // Set up an iron server
    let mut mount   = Mount::new();
    let mut web_ui  = UiHandler::<FlowBetweenSession>::from_sessions(sessions.clone());
    web_ui.set_websocket_port(WS_SERVER_PORT);
    mount.mount("/", flowbetween_static_files());
    mount.mount("/flowbetween/session", web_ui);
    
    // Run the WS server
    thread::spawn(move || {
        // Set up a websockets server
        let mut tokio_core  = Core::new().unwrap();
        let ws_handle       = Arc::new(tokio_core.handle());
        let ws_handler      = WebSocketHandler::from_sessions(sessions.clone());
        let ws_stream       = ws_handler.create_server(&format!("{}:{}", BIND_ADDRESS, WS_SERVER_PORT), ws_handle);

        println!("{} v{} preparing to serve websocket requests at {}", PACKAGE_NAME, PACKAGE_VERSION, format!("{}:{}", BIND_ADDRESS, WS_SERVER_PORT));
        tokio_core.run(ws_stream).unwrap();
    });

    // Run the iron server
    println!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, &format!("{}:{}", BIND_ADDRESS, SERVER_PORT));

    Iron::new(mount).http(&format!("{}:{}", BIND_ADDRESS, SERVER_PORT)).unwrap();
}
