use super::sessions::*;

use ui::*;
use websocket::*;
use websocket::async::{Server, TcpListener};
use websocket::server::{WsServer, NoTlsAcceptor};
use tokio_core::reactor;

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

    ///
    /// Creates a websocket. Bind address should be something like '127.0.0.1:3001'
    /// 
    pub fn create_server(&self, bind_address: &str, tokio_core_handle: Arc<reactor::Handle>) -> WsServer<NoTlsAcceptor, TcpListener> {
        // Bind a server
        let server = Server::bind(bind_address, &tokio_core_handle).unwrap();

        server
    }
}
