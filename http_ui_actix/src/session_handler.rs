use flo_http_ui::*;

use actix_web::*;
use actix_web::http::*;
use actix_web::dev::{Handler, AsyncResult};
use futures::*;
use futures::future;

use std::sync::*;

///
/// Creates the handler for an actix UI session
/// 
pub fn session_handler<CoreController: 'static+HttpController>() -> impl Handler<Arc<WebSessions<CoreController>>> {
    |req: HttpRequest<Arc<WebSessions<CoreController>>>| {
        match req.method() {
            &Method::POST => {
                // POST requests are used to send instructions to sessions

                // Request must contain a JSON body
                let result = Json::<UiHandlerRequest>::extract(&req)
                    .then(move |request| {
                        match request {
                            Ok(request) => {
                                // JSON data is valid: process this UI request

                                // Request not implemented
                                future::ok(req.build_response(StatusCode::NOT_FOUND).body("Not implemented"))
                            },

                            Err(_err) => {
                                // Failed to parse the JSON request for some reason
                                future::ok(req.build_response(StatusCode::BAD_REQUEST).body("FlowBetween session request is not in the expected format"))
                            }
                        }
                    });
                
                // Request will be ready in the future
                AsyncResult::async(Box::new(result))
            },

            &Method::GET => {
                // Get requests to sessions are used to retrieve the current state of various resources

                // (TODO!)
                AsyncResult::async(Box::new(future::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"))))
            },

            _ => {
                // Other requests are not supported
                AsyncResult::async(Box::new(future::ok(req.build_response(StatusCode::METHOD_NOT_ALLOWED).body("Method not allowed"))))
            }
        }
    }
}
