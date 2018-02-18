use super::sessions::*;

use ui::*;
use websocket::*;
use websocket::async::{Server, TcpStream};
use websocket::server::{InvalidConnection};
use websocket::server::upgrade::WsUpgrade;
use tokio_core::reactor;
use hyper::uri::*;
use bytes::BytesMut;

use futures::*;

use std::sync::*;
use std::net::SocketAddr;

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
    /// Handles incoming requests on a websocket connection
    /// 
    pub fn handle_incoming_requests(&self, incoming: Box<Stream<Item=(WsUpgrade<TcpStream, BytesMut>, SocketAddr), Error=InvalidConnection<TcpStream, BytesMut>>>, tokio_core_handle: Arc<reactor::Handle>) -> Box<Future<Item=(), Error=()>> {
        // Server will use our sessions object
        let sessions    = Arc::clone(&self.sessions);

        // Handle incoming requests
        let handle_requests = incoming
            .map_err(|InvalidConnection { error, ..}| error)
            .for_each(move |(upgrade, addr)| {
                // Only want connections for the rust-websocket protocol
                if !upgrade.protocols().iter().any(|protocol| protocol == "flo") {
                    // Reject anything that doesn't support it
                    tokio_core_handle.spawn(upgrade.reject().map_err(|_| ()).map(|_| ()));
                    return Ok(());
                }

                let (method, uri) = upgrade.request.subject.clone();
                println!("Websocket connection on {} to {}", addr, uri);

                // Accept websocket upgrades if the protocol is supported
                let handle_request = upgrade
                    .use_protocol("flo")
                    .accept()
                    .and_then(|(sink, _stream)| sink.send(Message::text("Hello, world").into()));

                // Spawn our request handler and be done
                let handle_request = handle_request
                    .map_err(|_err| ())
                    .map(|_| ());
                tokio_core_handle.spawn(handle_request);
                Ok(())
            })
            .then(|i| {
                match i {
                    Ok(())      => Ok(()),
                    Err(err)   => {
                        // TODO: errors (like just connecting via the browser) stop the server :-(
                        println!("WS err: {:?}", err);
                        Ok(())
                    }
                }
            });

        // Suppress errors and return the result
        // TODO: probably want to log errors instead but there's no logging framework yet
        let handle_requests = handle_requests.map_err(|_err: ()| ());
        Box::new(handle_requests)
    }

    ///
    /// Creates a websocket. Bind address should be something like '127.0.0.1:3001'
    /// 
    pub fn create_server(&self, bind_address: &str, tokio_core_handle: Arc<reactor::Handle>) -> Box<Future<Item=(), Error=()>> {
        // Bind a server
        let server      = Server::bind(bind_address, &tokio_core_handle).unwrap();

        // Register to handle requests on it
        self.handle_incoming_requests(server.incoming(), tokio_core_handle)
    }
}
