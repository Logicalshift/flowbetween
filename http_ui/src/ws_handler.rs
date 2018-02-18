use super::sessions::*;

use ui::*;
use websocket::*;

use std::sync::*;

///
/// Represents a handler for connections to a session using websockets
/// 
pub struct WebSocketHandler<CoreController: Controller> {
    /// The sessions that will be served via websocket(s)
    sessions: Arc<WebSessions<CoreController>>
}

impl<CoreController: Controller+'static> WebSocketHandler<CoreController> {
    ///
    /// Creates a new websocket handler
    ///
    pub fn new() -> WebSocketHandler<CoreController> {
        WebSocketHandler { sessions: Arc::new(WebSessions::new()) }
    }

    ///
    /// Creates a websocket handler that will provide websockets for a pre-set
    /// set of sessions
    /// 
    pub fn from_sessions(sessions: Arc<WebSessions<CoreController>>) -> WebSocketHandler<CoreController> {
        WebSocketHandler { sessions: sessions }
    }
}
