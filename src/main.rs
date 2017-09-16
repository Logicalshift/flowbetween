//!
//! # FlowBetween HTTP server
//!

extern crate iron;
extern crate static_files;

const PACKAGE_NAME: &str    = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVER_ADDR: &str     = "127.0.0.1:3000";

fn main() {
    println!("{} v{} now serving requests at {}", PACKAGE_NAME, PACKAGE_VERSION, SERVER_ADDR);
}
