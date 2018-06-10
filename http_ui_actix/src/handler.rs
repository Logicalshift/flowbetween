use flo_http_ui::*;

use actix_web::*;
use actix_web::dev::Handler;

use std::sync::*;

///
/// Creates the handler for an actix UI session
/// 
pub fn create_handler<CoreController: HttpController>() -> impl Handler<Arc<WebSessions<CoreController>>> {
    |req| {
        "Hello, world"
    }
}
