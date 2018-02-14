use super::event::*;
use super::update::*;
use super::http_session::*;
use super::http_controller::*;
use super::http_user_interface::*;

// TODO: only used for the response/request structures
use super::ui_handler::*;

use ui::session::*;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;
use mount::*;

use uuid::*;
use serde_json;
use bodyparser::*;
use futures::executor;

use std::sync::*;
use std::collections::*;

///
/// Handles creating and maintainng HTTP sessions
/// 
pub struct UiHandler2<CoreController: HttpController> {
    sessions: Mutex<HashMap<String, Arc<Mutex<HttpSession<UiSession<CoreController>>>>>>
}

impl<CoreController: HttpController+'static> UiHandler2<CoreController> {
    ///
    /// Creates a new UI handler
    /// 
    pub fn new() -> UiHandler2<CoreController> {
        UiHandler2 {
            sessions: Mutex::new(HashMap::new())
        }
    }

    ///
    /// Returns the base URL for a request
    ///
    fn base_url(req: &Request) -> Url {
        // Get the original URL for this request
        let original_url = req.extensions.get::<OriginalUrl>()
            .map(|url| url.clone())
            .unwrap_or(Url::parse("http://localhost/").unwrap());
        
        // Also need the request url
        let request_url     = req.url.clone();

        // Request URL path will be the last part of the original URL: remove enough parts that 
        let original_path       = original_url.path();
        let request_path        = request_url.path();

        let original_path_len   = original_path.len();
        let request_path_len    = {
            if request_path.len() == 1 && request_path[0] == "" {
                0
            } else if request_path.len() > original_path.len() {
                0
            } else {
                request_path.len()
            }
        };

        let base_path           = original_url.path()[0..(original_path_len-request_path_len)].join("/");

        let mut base_url = original_url.clone();
        base_url.as_mut().set_path(&base_path);

        base_url
    }

    ///
    /// Creates a new session and session state, returning the ID
    ///
    pub fn new_session(&self, base_url: &str) -> String {
        // Generate an ID for this session
        let session_id          = Uuid::new_v4().simple().to_string();

        // Produce the URI for this session
        let session_uri         = format!("{}/{}", base_url, session_id);

        // Create the session controller
        let session_controller  = CoreController::start_new();

        // Turn into a HttpSession by starting up a UI session
        let ui                  = UiSession::new(session_controller);
        let http_ui             = HttpUserInterface::new(Arc::new(ui), session_uri);
        let session             = HttpSession::new(Arc::new(http_ui));

        // Store in the list of active sessions
        self.sessions.lock().unwrap().insert(session_id.clone(), Arc::new(Mutex::new(session)));

        // Result is the session ID
        session_id
    }

    ///
    /// Fills in a response structure for a request with no session
    ///
    fn handle_no_session(&self, base_url: &str, response: &mut UiHandlerResponse, req: &UiHandlerRequest) {
        for event in req.events.iter() {
            match event.clone() {
                // When there is no session, we can request that one be created
                Event::NewSession => {
                    let session_id = self.new_session(base_url);
                    response.updates.push(Update::NewSession(session_id));
                },

                // For any other event, a session is required, so we add a 'missing session' notification to the response
                _ => response.updates.push(Update::MissingSession)
            }
        }
    }

    ///
    /// Sends a request to a session
    /// 
    fn handle_with_session(&self, session: &mut HttpSession<UiSession<CoreController>>, response: &mut UiHandlerResponse, req: UiHandlerRequest) {
        // Send the events to the session
        let handle_result = session.send_events(req.events);

        // Wait for the response
        let update_results = executor::spawn(handle_result).wait_future().unwrap();

        // Add to the response
        response.updates.extend(update_results);
    }

    ///
    /// Handles a UI handler request
    ///
    pub fn handle_ui_request(&self, req: UiHandlerRequest, base_url: &str) -> Response {
        // The response that we'll return for this request
        let mut response    = UiHandlerResponse { updates: vec![] };
        let session_id      = req.session_id.clone();

        // Dispatch depending on whether or not this request corresponds to an active session
        match session_id {
            None                    => self.handle_no_session(base_url, &mut response, &req),
            Some(ref session_id)    => {
                // Try to fetch the session for this ID
                let session = {
                    self.sessions.lock().unwrap()
                        .get(session_id)
                        .cloned()
                };

                // If the session ID is not presently registered, then we proceed as if the session is missing 
                match session {
                    Some(session) => {
                        let mut session = session.lock().unwrap();
                        self.handle_with_session(&mut session, &mut response, req)
                    },
                    _ => 
                        self.handle_no_session(base_url, &mut response, &req)
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

impl<CoreController: HttpController+'static> Handler for UiHandler2<CoreController> {
    ///
    /// Handles a request for a UI session (or creates new sessions)
    ///
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.method {
            Method::Post => {
                let is_json         = match req.headers.get() { Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, _))) => true, _ => false };
                let mut base_url    = Self::base_url(req).path().join("/");

                if base_url.chars().nth(0) != Some('/') {
                    base_url.insert(0, '/');
                }

                if !is_json {
                    // Must be a JSON POST request
                    Ok(Response::with((status::BadRequest)))
                } else {
                    // Parse the request
                    let request = req.get::<Struct<UiHandlerRequest>>();

                    match request {
                        Ok(Some(request))   => Ok(self.handle_ui_request(request, &base_url)),
                        Ok(None)            => Ok(Response::with((status::BadRequest))),
                        Err(_)              => Ok(Response::with((status::BadRequest)))
                    }
                }
            },

            /*
            Method::Get => {
                // Resource fetch
                Ok(self.handle_resource_request(req))
            },
            */

            _ => {
                // Unsupported method
                Ok(Response::with((status::BadRequest)))
            }
        }
    }
}
