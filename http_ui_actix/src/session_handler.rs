use super::actix_session::*;
use super::session_resource_handler::*;

use flo_http_ui::*;

use actix_web::*;
use actix_web::http::*;
use actix_web::dev::{Handler, AsyncResult};
use actix_web::Error;
use futures::*;
use futures::future;
use futures::stream;

use std::sync::*;

///
/// Retrieves the base URL for a request
/// 
fn base_url<TState>(req: &HttpRequest<TState>) -> String {
    let full_url    = req.uri().path();
    let tail        = req.match_info().get("tail").unwrap_or("");
    let base_len    = full_url.len() - tail.len();

    if tail == "/" {
        full_url.to_string()
    } else {
        full_url[0..base_len].to_string()
    }
}

///
/// Handles a request with a session that exists
/// 
fn handle_with_session<Session: ActixSession>(session: &mut HttpSession<Session::CoreUi>, ui_request: &UiHandlerRequest) -> impl Future<Item=UiHandlerResponse, Error=Error> {
    // Send the events followed by a tick
    let mut events  = ui_request.events.clone();
    events.push(Event::Tick);

    // Send the events to get the updates
    let updates     = session.send_events(events);

    // Turn the updates into a response
    updates
        .map(|updates| UiHandlerResponse { updates })
        .map_err(|_| unimplemented!())
}

///
/// Handles a request for a session that doesn't exist
/// 
/// (This creates a new session for this user)
/// 
fn handle_no_session<Session: ActixSession>(session: Arc<Session>, base_url: String, ui_request: &UiHandlerRequest) -> impl Future<Item=UiHandlerResponse, Error=Error> {
    // Convert the events into an iterator
    let ui_request  = ui_request.clone();
    let events      = stream::iter_ok::<_, Error>(ui_request.events.into_iter());

    // Map each event onto the corresponding update
    let updates     = events
        .map(move |event| {
            match event {
                Event::NewSession => {
                    let mut updates = vec![];

                    // Start a new session
                    let session_controller  = Session::Controller::start_new();
                    let session_id          = session.new_session(session_controller, &base_url);

                    // Return the new session ID
                    updates.push(Update::NewSession(session_id));

                    // TODO: With Actix, we run a websocket on the same port, so also send a websocket update

                    // Return the updates
                    stream::iter_ok(updates)             
                },

                // For any other event, a session is required, so we indicate that it's missing
                _ => stream::iter_ok(vec![Update::MissingSession])
            }
        })
        .flatten();

    // Turn the stream into a response
    let response = updates
        .collect()
        .map(|updates| UiHandlerResponse { updates });

    // This is the future we return
    response
}

///
/// Handles a JSON UI request
/// 
fn handle_ui_request<Session: ActixSession+'static>(req: HttpRequest<Arc<Session>>, ui_request: &UiHandlerRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    // Fetch the session ID from the request
    let session_id  = ui_request.session_id.clone();

    // Generate the response
    let response: Box<Future<Item=UiHandlerResponse, Error=Error>> = match session_id {
        None                => Box::new(handle_no_session(Arc::clone(req.state()), base_url(&req), ui_request)),
        Some(session_id)    => {
            // Try to fetch the session corresponding to this ID
            let session = req.state().get_session(&session_id);

            // Send the events to the appropriate session if we find one
            match session {
                Some(session)   => Box::new(handle_with_session::<Session>(&mut *session.lock().unwrap(), ui_request)),
                None            => Box::new(handle_no_session(Arc::clone(req.state()), base_url(&req), ui_request))
            }
        }
    };
    
    // Turn the UI response into a JSON response
    response
        .map(move |response| {
            req.build_response(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "application/json; charset=utf8")
                .json(response)
        })
}

///
/// Creates the handler for an actix UI session
/// 
pub fn session_handler<Session: 'static+ActixSession>() -> impl Handler<Arc<Session>> {
    // Create the session resource handler
    let get_handler = Mutex::new(session_resource_handler());

    // Wrap into a function
    move |req: HttpRequest<Arc<Session>>| {
        match req.method() {
            &Method::POST => {
                // POST requests are used to send instructions to sessions

                // Request must contain a JSON body
                let result = Json::<UiHandlerRequest>::extract(&req)
                    .then(move |ui_request| -> Box<Future<Item=HttpResponse, Error=Error>> {
                        match ui_request {
                            Ok(ui_request) => {
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
                // Get requests are handled by the session resource handler
                let resource_request    = get_handler.lock().unwrap().handle(req.clone());
                let response            = resource_request.respond_to(&req);

                match response {
                    Ok(response)    => response.into(),
                    Err(err)        => AsyncResult::err(err)
                }
            },

            _ => {
                // Other requests are not supported
                AsyncResult::async(Box::new(future::ok(req.build_response(StatusCode::METHOD_NOT_ALLOWED).body("Method not allowed"))))
            }
        }
    }
}
