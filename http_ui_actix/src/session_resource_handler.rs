use super::actix_session::*;

use actix_web::*;
use actix_web::dev::{Handler};

use std::sync::*;

///
/// Handler for get requests for a session
/// 
pub fn session_resource_handler<Session: ActixSession>() -> impl Handler<Arc<Session>> {
    |req: HttpRequest<Arc<Session>>| {
        // The path is the tail of the request
        let path    = req.match_info().get("tail");
        let state   = Arc::clone(req.state());

        if let Some(path) = path {
            // Path is valid
        } else {
            // No tail path was supplied (likely this handler is being called from the wrong place)
        }
        
        "Resource"
    }
}
