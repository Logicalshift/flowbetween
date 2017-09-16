//!
//! # FlowBetween HTTP server
//!

extern crate hyper;
extern crate futures;
extern crate static_service;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVER_ADDR: &str     = "127.0.0.1:3000";

struct HelloWorld;

const PHRASE: &'static str = "Hello, World!";
impl Service for HelloWorld {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE)
        )
    }
}

fn main() {
    let addr    = SERVER_ADDR.parse().unwrap();
    let server  = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();

    println!("{} v{} now serving requests at {}", PACKAGE_NAME, PACKAGE_VERSION, SERVER_ADDR);

    server.run().unwrap();
}
