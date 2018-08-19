use super::http_session::*;
use super::http_user_interface::*;

use ui::*;
use ui::session::*;

use flo_logging::*;
use uuid::*;

use std::sync::*;
use std::collections::*;

///
/// Manages the active sessions
/// 
pub struct WebSessions<CoreController: Controller> {
    log: LogPublisher,

    /// The sessions
    sessions: Mutex<HashMap<String, Arc<Mutex<HttpSession<UiSession<CoreController>>>>>>
}

impl<CoreController: Controller+'static> WebSessions<CoreController> {
    ///
    /// Creates a new websessions object
    /// 
    pub fn new() -> WebSessions<CoreController> {
        WebSessions {
            log:        LogPublisher::new(module_path!()),
            sessions:   Mutex::new(HashMap::new())
        }
    }

    ///
    /// Creates a new session and returns its ID
    /// 
    pub fn new_session(&self, controller: CoreController, base_path: &str) -> String {
        // Generate a session ID using the UUID library
        let session_id          = Uuid::new_v4().simple().to_string();

        self.log.log((Level::Info, format!("Starting session ID {}", session_id)));

        // Produce the URI for this session
        let session_uri         = format!("{}/{}", base_path, session_id);

        // Generate the varioud components of the session
        let ui_session      = UiSession::new(controller);
        let http_ui         = HttpUserInterface::new(Arc::new(ui_session), session_uri);
        let http_session    = HttpSession::new(Arc::new(http_ui));

        // Store the new session and associate it with this ID
        self.sessions.lock().unwrap().insert(session_id.clone(), Arc::new(Mutex::new(http_session)));

        // Return the session
        session_id
    }

    ///
    /// Retrieves the session with the specified ID form this object
    /// 
    pub fn get_session(&self, session_id: &str) -> Option<Arc<Mutex<HttpSession<UiSession<CoreController>>>>> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }
}
