use std::io::*;
use std::sync::*;
use std::collections::*;

use super::event::*;
use super::update::*;
use super::session::*;
use super::htmlcontrol::*;
use super::session_state::*;

extern crate serde_json;
use serde::*;
use serde_json::*;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;

use bodyparser::*;

///
/// Handler that runs a particular UI through the HTTP interface
///
pub struct UiHandler<TSession: Session> {
    /// The sessions that are currently active for this handler
    active_sessions: Mutex<HashMap<String, (Arc<SessionState>, TSession)>>,
}

impl<TSession: Session> UiHandler<TSession> {
    ///
    /// Creates a new UI handler
    ///
    pub fn new() -> UiHandler<TSession> {
        UiHandler { 
            active_sessions: Mutex::new(HashMap::new()),  
        }
    }

    ///
    /// Creates a new session and session state, returning the ID
    ///
    pub fn new_session(&self) -> String {
        // Generate a new session
        let new_state               = Arc::new(SessionState::new());
        let new_session: TSession   = TSession::start_new(new_state.clone());

        // Store in the list of active sessions
        let mut active_sessions = self.active_sessions.lock().unwrap();
        active_sessions.insert(String::from(new_state.id()), (new_state.clone(), new_session));

        // Result is the session ID
        String::from(new_state.id())
    }

    ///
    /// Fills in a response structure for a request with no session
    ///
    fn handle_no_session(&self, response: &mut UiHandlerResponse, req: &UiHandlerRequest) {
        unimplemented!()
    }

    ///
    /// Dispatches a response structure to a session
    ///
    fn handle_with_session(&self, state: Arc<SessionState>, session: &mut TSession, response: &mut UiHandlerResponse, req: &UiHandlerRequest) {
        unimplemented!()
    }

    ///
    /// Handles a UI handler request
    ///
    pub fn handle_request(&self, req: &UiHandlerRequest) -> Response {
        // The response that we'll return for this request
        let mut response = UiHandlerResponse { updates: vec![] };

        // Dispatch depending on whether or not this request corresponds to an active session
        match req.session_id {
            None                    => self.handle_no_session(&mut response, req),
            Some(ref session_id)    => {
                // Try to fetch the session for this ID
                let mut active_sessions = self.active_sessions.lock().unwrap();
                let session             = active_sessions.get_mut(session_id);

                // If the session ID is not presently registered, then we proceed as if the session is missing 
                match session {
                    Some(&mut (ref mut session_state, ref mut session)) => 
                        self.handle_with_session(session_state.clone(), session, &mut response, req),
                    _ => 
                        self.handle_no_session(&mut response, req)
                }
            }
        };

        // Generate the final response
        Response::with((
            status::Ok,
            Header(ContentType::json()),
            serde_json::to_string(&response).unwrap()
        ))
    }
}

///
/// Structure of a request sent to the UI handler
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerRequest {
    /// The session ID, if there is one
    session_id: Option<String>,

    /// The events that the UI wishes to report with this request
    events: Vec<Event>
}

///
/// Structure of a UI handler response
///
#[derive(Clone, Serialize, Deserialize)]
pub struct UiHandlerResponse {
    /// Updates generated for this request
    updates: Vec<Update>
}

impl<TSession: Session+'static> Handler for UiHandler<TSession> {
    ///
    /// Handles a request for a UI session (or creates new sessions)
    ///
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let is_post = req.method == Method::Post;
        let is_json = match req.headers.get() { Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, _))) => true, _ => false };

        if !is_post || !is_json {
            // Must be a JSON POST request
            Ok(Response::with((status::BadRequest)))
        } else {
            // Parse the request
            let request = req.get::<Struct<UiHandlerRequest>>();

            match request {
                Ok(Some(request))   => Ok(self.handle_request(&request)),
                Ok(None)            => Ok(Response::with((status::BadRequest))),
                Err(_)              => Ok(Response::with((status::BadRequest)))
            }
        }
    }
}