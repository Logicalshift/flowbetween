use flo_http_ui::*;

use actix_web::*;
use actix_web::dev::Handler;

use std::sync::*;

///
/// Creates the handler for an actix UI session
/// 
pub fn session_handler<CoreController: HttpController>() -> impl Handler<Arc<WebSessions<CoreController>>> {
    |req: HttpRequest<Arc<WebSessions<CoreController>>>| {
        println!("{:?} {:?}", req.path(), req.match_info().get("tail"));

        "Hello, world"
    }
}
