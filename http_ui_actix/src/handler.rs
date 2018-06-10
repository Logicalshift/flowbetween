use actix_web::*;
use actix_web::dev::Handler;

///
/// Creates the handler for an actix UI session
/// 
pub fn create_handler() -> impl Handler<()> {
    |req| {
        "Hello, world"
    }
}

