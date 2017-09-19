use std::sync::*;
use std::collections::*;

use super::session::*;
use super::session_state::*;
use super::htmlcontrol::*;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;

///
/// Handler that runs a particular UI through the HTTP interface
///
pub struct UiHandler<TSession: Session> {
    /// The sessions that are currently active for this handler
    active_sessions: Mutex<HashMap<String, TSession>>
}

impl<TSession: Session> UiHandler<TSession> {
    pub fn new() -> UiHandler<TSession> {
        UiHandler { active_sessions: Mutex::new(HashMap::new()) }
    }
}

impl<TSession: Session+'static> Handler for UiHandler<TSession> {
    ///
    /// Handles a request for a UI session (or creates new sessions)
    ///
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let new_state               = Arc::new(SessionState::new());
        let new_session: TSession   = TSession::start_new(new_state.clone());

        unimplemented!()
    }
}