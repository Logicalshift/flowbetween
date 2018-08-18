//!
//! # FlowBetween HTTP server
//!
#![warn(bare_trait_objects)]

#[cfg(feature="gtk")]   extern crate flo_gtk_ui;
#[cfg(feature="http")]  extern crate flo_http_ui;
#[cfg(feature="http")]  extern crate flo_http_ui_actix;
#[cfg(feature="http")]  extern crate actix_web;

extern crate flo_ui;
extern crate flo_ui_files;
extern crate flo;
extern crate flo_binding;
extern crate flo_animation;
extern crate flo_anim_sqlite;
extern crate flo_logging;

extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate pretty_env_logger;

mod flo_session;
#[cfg(feature="http")]  mod http_session;

#[cfg(feature="http")]  use actix_web as aw;

use std::sync::*;
use std::thread;
use std::thread::JoinHandle;

use flo_logging::*;

#[cfg(feature="http")]  use flo_http_ui::*;
#[cfg(feature="http")]  use flo_http_ui_actix as flo_actix;
#[cfg(feature="gtk")]   use flo_gtk_ui::*;

use self::flo_session::*;

#[cfg(feature="http")]  const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
#[cfg(feature="http")]  const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature="http")]  const SERVER_PORT: u32      = 3000;
#[cfg(feature="http")]  const BIND_ADDRESS: &str    = "0.0.0.0";

#[cfg(feature="http")]
fn main_actix() -> Option<JoinHandle<()>> {
    Some(thread::spawn(|| {
        // Create the web session structure
        let sessions: Arc<WebSessions<FlowBetweenSession>> = Arc::new(WebSessions::new());

        // Log that we're getting ready
        log().log(format!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, &format!("{}:{}", BIND_ADDRESS, SERVER_PORT)));

        // Run the actix server
        aw::server::new(move || {
                aw::App::with_state(sessions.clone())
                    .handler("/flowbetween/session", flo_actix::session_handler())
                    .handler("/ws", flo_actix::session_websocket_handler())
                    .handler("/", flo_actix::flowbetween_static_file_handler())
            })
            .bind(&format!("{}:{}", BIND_ADDRESS, SERVER_PORT))
            .unwrap()
            .run();
    }))
}

#[cfg(not(feature="http"))]
fn main_actix() -> Option<JoinHandle<()>> {
    None
}

#[cfg(feature="gtk")]
fn main_gtk() -> Option<JoinHandle<()>> {
    Some(thread::spawn(|| {
        // Create a GTK session
        let gtk_ui      = GtkUserInterface::new();
        let gtk_session = GtkSession::from(FlowBetweenSession::new(), gtk_ui);

        gtk_session.run();
    }))
}

#[cfg(not(feature="gtk"))]
fn main_gtk() -> Option<JoinHandle<()>> {
    None
}

#[cfg(not(any(feature="gtk", feature="http")))]
compile_error!("You must pick a UI implementation as a feature to compile FlowBetween (cargo build scripts cannot autodetect, sadly). Build with cargo --features gtk,http");

fn main() {
    // Set up logging
    pretty_env_logger::init();

    // TODO: be a bit more sensible about this (right now this is just the GTK version shoved onto the start of the HTTP version)

    let gtk_thread      = main_gtk();
    let actix_thread    = main_actix();

    if let Some(gtk_thread) = gtk_thread {
        // If there's a GTK thread, then we stop when it stops
        gtk_thread.join().unwrap();
    } 
    
    if let Some(actix_thread) = actix_thread {
        // Otherwise we monitor the HTTP thread, if it exists
        actix_thread.join().unwrap()
    }
}
