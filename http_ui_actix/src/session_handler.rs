use super::actix_session::*;

use flo_http_ui::*;

use actix_web::*;
use actix_web::http::*;
use actix_web::dev::{Handler, AsyncResult};
use actix_web::Error;
use futures::*;
use futures::future;

use std::sync::*;

///
/// Handles a JSON UI request
/// 
fn handle_ui_request<Session: ActixSession>(req: HttpRequest<Arc<Session>>, ui_request: &UiHandlerRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    future::ok(req.build_response(StatusCode::NOT_FOUND).body("Not implemented yet"))
}

///
/// Creates the handler for an actix UI session
/// 
pub fn session_handler<Session: 'static+ActixSession>() -> impl Handler<Arc<Session>> {
    |req: HttpRequest<Arc<Session>>| {
        match req.method() {
            &Method::POST => {
                // POST requests are used to send instructions to sessions

                // Request must contain a JSON body
                let result = Json::<UiHandlerRequest>::extract(&req)
                    .then(move |ui_request| -> Box<Future<Item=HttpResponse, Error=Error>> {
                        match ui_request {
                            Ok(ui_request) => {
                                // JSON data is valid: process this UI request
                                println!("{:?}", ui_request);

                                // Process this UI request
                                Box::new(handle_ui_request(req, &*ui_request))
                            },

                            Err(_err) => {
                                // Failed to parse the JSON request for some reason
                                Box::new(future::ok(req.build_response(StatusCode::BAD_REQUEST).body("FlowBetween session request is not in the expected format")))
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
