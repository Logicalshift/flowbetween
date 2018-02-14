use super::event::*;
use super::update::*;
use super::http_session::*;
use super::http_controller::*;
use super::http_user_interface::*;

// TODO: only used for the response/request structures
use super::ui_handler::*;

use ui::*;
use ui::session::*;

use iron::*;
use iron::mime::*;
use iron::method::*;
use iron::headers::*;
use iron::modifiers::*;
use mount::*;

use uuid::*;
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
}