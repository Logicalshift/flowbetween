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
extern crate flo_canvas;
extern crate flo;
extern crate flo_binding;
extern crate flo_animation;
extern crate flo_sqlite_storage;
extern crate flo_logging;

extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate log;
extern crate pretty_env_logger;

mod logo_controller;
mod flo_session;
#[cfg(feature="http")]  mod http_session;

#[cfg(feature="http")]  use actix_web as aw;
#[cfg(feature="http")]  use actix_web::web as web;

use std::sync::*;
use std::thread;
use std::thread::JoinHandle;

use log::*;
use flo_logging::*;

#[cfg(feature="http")]  use flo_http_ui::*;
#[cfg(feature="http")]  use flo_http_ui_actix as flo_actix;
#[cfg(feature="http")]  use actix_rt;
#[cfg(feature="gtk")]   use flo_ui::session::*;
#[cfg(feature="gtk")]   use flo_gtk_ui::*;
#[cfg(feature="gtk")]   use futures::executor;
#[cfg(feature="gtk")]   use futures::prelude::*;

use self::flo_session::*;

#[cfg(feature="http")]  const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
#[cfg(feature="http")]  const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature="http")]  const SERVER_PORT: u32      = 3000;
#[cfg(feature="http")]  const BIND_ADDRESS: &str    = "0.0.0.0";

#[cfg(feature="http")]
fn main_actix() -> Option<JoinHandle<()>> {
    Some(thread::spawn(|| {
        actix_rt::System::new("FlowBetween").block_on(async {
            let log = LogPublisher::new("main_actix");

            // Create the web session structure
            let sessions: Arc<WebSessions<FlowBetweenSession>> = Arc::new(WebSessions::new());

            // Log that we're getting ready
            log.log(format!("{} v{} preparing to serve requests at {}", PACKAGE_NAME, PACKAGE_VERSION, &format!("{}:{}", BIND_ADDRESS, SERVER_PORT)));

            // Start the actix server
            aw::HttpServer::new(move || {
                    // Something in Actix's type system involving the private Factory type is unhappy with using the static file handler function directlyh
                    // (Error is unhelpful but I think it's to do with if the function can be cloned or not)
                    let static_file_handler = Arc::new(flo_actix::flowbetween_static_file_handler());

                    aw::App::new()
                        .app_data(sessions.clone())
                        .service(web::resource("/flowbetween/session")
                            .route(web::get().to(flo_actix::session_get_handler::<WebSessions<FlowBetweenSession>>))
                            .route(web::post().to(flo_actix::session_post_handler::<WebSessions<FlowBetweenSession>>)))
                        .service(web::resource("/flowbetween/session/{tail:.*}")
                            .route(web::get().to(flo_actix::session_get_handler::<WebSessions<FlowBetweenSession>>))
                            .route(web::post().to(flo_actix::session_post_handler::<WebSessions<FlowBetweenSession>>)))
                        .service(web::resource("/ws").route(web::to(flo_actix::session_websocket_handler::<WebSessions<FlowBetweenSession>>)))
                        .service(web::resource("/ws/{tail:.*}").route(web::to(flo_actix::session_websocket_handler::<WebSessions<FlowBetweenSession>>)))
                        .service(web::resource("/{tail:.*}").route(web::to(move |r| static_file_handler(r))))
                })
                .bind(&format!("{}:{}", BIND_ADDRESS, SERVER_PORT))
                .expect("Failed to bind HTTP server to port")
                .run()
                .await
                .expect("Http server failed while running");
        });
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
        let (gtk_ui, gtk_run_loop)  = GtkUserInterface::new();
        let (session, ui_run_loop)  = UiSession::new(FlowBetweenSession::new());
        let gtk_session             = GtkSession::new(session, gtk_ui);

        let run_session             = gtk_session.run();
        let run_loop                = future::select(gtk_run_loop.boxed(), ui_run_loop.boxed());
        let run_loop                = future::select(run_session.boxed_local(), run_loop);

        // Run on this thread
        executor::block_on(async {
            run_loop.await;
        })
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
    send_logs_to(Box::new(pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .filter_module("mio", LevelFilter::Warn)
        .filter_module("tokio_reactor", LevelFilter::Warn)
        .filter_module("actix_web", LevelFilter::Info)
        .build()));
    send_rust_logs_to_flo_logs().unwrap();

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
