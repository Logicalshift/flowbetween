use super::actix_session::*;
use super::session_resource_handler::*;

use flo_http_ui::*;

use actix_web::*;
use actix_web::web::Json;
use actix_web::Error;
use futures::*;
use futures::stream;
use futures::future::{BoxFuture, LocalBoxFuture};
use futures::executor::{ThreadPool};

use std::sync::*;

lazy_static! {
    /// The thread pool where the sessions will run
    static ref THREAD_POOL: ThreadPool = ThreadPool::new().unwrap();
}

///
/// Retrieves the base URL for a request
///
fn base_url(req: &HttpRequest) -> String {
    let full_url    = req.uri().path();
    let tail        = req.match_info().get("tail").unwrap_or("");
    let base_len    = full_url.len() - tail.len();

    if tail == "/" || tail == full_url {
        full_url.to_string()
    } else {
        full_url[0..base_len].to_string()
    }
}

///
/// Handles a request with a session that exists
///
fn handle_with_session<Session: ActixSession>(session: &mut HttpSession<Session::CoreUi>, ui_request: &UiHandlerRequest) -> impl Future<Output=Result<UiHandlerResponse, Error>> {
    // Send the events followed by a tick
    let mut events  = ui_request.events.clone();
    events.push(Event::Tick);

    // Send the events to get the updates
    let updates     = session.send_events(events);

    // Turn the updates into a response
    updates
        .map(|updates| Ok(UiHandlerResponse { updates }))
}

///
/// Handles a request for a session that doesn't exist
///
/// (This creates a new session for this user)
///
fn handle_no_session<Session: ActixSession>(session: Arc<Session>, base_url: String, ui_request: &UiHandlerRequest) -> impl Future<Output=Result<UiHandlerResponse, Error>> {
    // Convert the events into an iterator
    let ui_request  = ui_request.clone();
    let events      = stream::iter::<_>(ui_request.events.into_iter());

    // Map each event onto the corresponding update
    let updates     = events
        .map(move |event| {
            match event {
                Event::NewSession => {
                    let mut updates = vec![];

                    // Start a new session
                    let session_controller      = Session::Controller::start_new();
                    let (session_id, run_loop)  = session.new_session(session_controller, &base_url);

                    // Spawn the session run loop
                    THREAD_POOL.spawn_ok(run_loop);

                    // Return the new session ID
                    updates.push(Update::NewSession(session_id));

                    // Websocket runs on the same port as the main session
                    updates.push(Update::WebsocketSamePort);

                    // TODO: With Actix, we run a websocket on the same port, so also send a websocket update

                    // Return the updates
                    stream::iter(updates)
                },

                // For any other event, a session is required, so we indicate that it's missing
                _ => stream::iter(vec![Update::MissingSession])
            }
        })
        .flatten();

    // Turn the stream into a response
    let response = updates
        .collect()
        .map(|updates| Ok(UiHandlerResponse { updates }));

    // This is the future we return
    response
}

///
/// Handles a JSON UI request
///
fn handle_ui_request<Session: ActixSession+Sync+Send+'static>(req: HttpRequest, ui_request: &UiHandlerRequest) -> impl Future<Output=Result<HttpResponse, Error>> {
    let session_state = req.app_data::<Arc<Session>>().expect("Flowbetween session state");

    let session_state = Arc::clone(&session_state);

    // Fetch the session ID from the request
    let session_id  = ui_request.session_id.clone();

    // Generate the response
    let response: BoxFuture<'static, Result<UiHandlerResponse, Error>> = match session_id {
        None                => Box::pin(handle_no_session(session_state, base_url(&req), ui_request)),
        Some(session_id)    => {
            // Try to fetch the session corresponding to this ID
            let session = session_state.get_session(&session_id);

            // Send the events to the appropriate session if we find one
            match session {
                Some(session)   => Box::pin(handle_with_session::<Session>(&mut *session.lock().unwrap(), ui_request)),
                None            => Box::pin(handle_no_session(session_state, base_url(&req), ui_request))
            }
        }
    };

    // Turn the UI response into a JSON response
    response
        .map(move |response| {
            response.map(|response| {
                HttpResponse::Ok()
                    .header(http::header::CONTENT_TYPE, "application/json; charset=utf-8")
                    .json(response)
                })
        })
}

///
/// Post request handler for the session URL
///
pub fn session_post_handler<Session: 'static+Send+Sync+ActixSession>(req: HttpRequest, ui_request: Json<UiHandlerRequest>) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    // Process this UI request
    Box::pin(handle_ui_request::<Session>(req, &*ui_request))
}

///
/// Get request handler for the session URL
///
pub fn session_get_handler<Session: 'static+ActixSession>(req: HttpRequest) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    // Get requests are handled by the session resource handler
    session_resource_handler::<Session>(req)
}
