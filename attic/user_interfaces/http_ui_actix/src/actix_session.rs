use flo_ui::*;
use flo_ui::session::*;
use flo_http_ui::*;
use flo_logging::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Trait implemented by objects that can work as an actix session
///
pub trait ActixSession {
    /// The controller type for this session
    type Controller: HttpController+Controller+'static;

    /// The core user interface for this session
    type CoreUi: CoreUserInterface+Send+Sync+'static;

    ///
    /// Creates a new session and returns its ID
    ///
    fn new_session(&self, controller: Self::Controller, base_path: &str) -> (String, BoxFuture<'static, ()>);

    ///
    /// Retrieves the session with the specified ID, if present
    ///
    fn get_session(&self, session_id: &str) -> Option<Arc<Mutex<HttpSession<Self::CoreUi>>>>;

    ///
    /// Retrieves the log for this session
    ///
    fn get_log(&self) -> &LogPublisher;
}

impl<CoreController: HttpController+Controller+'static> ActixSession for WebSessions<CoreController> {
    type Controller = CoreController;
    type CoreUi     = UiSession<CoreController>;

    ///
    /// Creates a new session and returns its ID
    ///
    #[inline]
    fn new_session(&self, controller: Self::Controller, base_path: &str) -> (String, BoxFuture<'static, ()>) {
        let (id, run_loop) = WebSessions::<CoreController>::new_session(self, controller, base_path);
        (id, run_loop.boxed())
    }

    ///
    /// Retrieves the session with the specified ID, if present
    ///
    #[inline]
    fn get_session(&self, session_id: &str) -> Option<Arc<Mutex<HttpSession<Self::CoreUi>>>> {
        WebSessions::<CoreController>::get_session(self, session_id)
    }

    ///
    /// Retrieves the log for this session
    ///
    fn get_log(&self) -> &LogPublisher {
        WebSessions::<CoreController>::get_log(self)
    }
}
