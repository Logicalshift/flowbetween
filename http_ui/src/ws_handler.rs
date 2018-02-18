use super::sessions::*;

use ui::*;
use serde_json;
use websocket::*;
use websocket::message::{OwnedMessage};
use websocket::async::{Server, TcpStream};
use websocket::server::{InvalidConnection};
use websocket::server::upgrade::WsUpgrade;
use tokio_core::reactor;
use hyper::uri::*;
use bytes::BytesMut;

use futures::*;
use futures::future;

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
                let sessions = sessions.clone();

                // Only want connections for the rust-websocket protocol
                if !upgrade.protocols().iter().any(|protocol| protocol == "flo") {
                    // Reject anything that doesn't support it
                    tokio_core_handle.spawn(upgrade.reject().map_err(|_| ()).map(|_| ()));
                    return Ok(());
                }

                let (_method, uri) = upgrade.request.subject.clone();
                println!("Websocket connection on {} to {}", addr, uri);

                // Attempt to fetch the session from the URI
                let mut session = match uri {
                    RequestUri::AbsolutePath(path)  => path,
                    RequestUri::AbsoluteUri(uri)    => uri.query().unwrap_or("").to_string(),
                    _                               => "".to_string()
                };

                // Remove the '/' from the start of the path
                if session.starts_with('/') {
                    session.remove(0);
                }

                // Attempt to retrieve the session with this ID
                let session = sessions.get_session(&session);

                match session {
                    Some(session) => {
                        // Get the event streams and sinks for this session
                        let http_ui = { session.lock().unwrap().http_ui() };
                        let events  = http_ui.get_input_sink();
                        let updates = http_ui.get_updates();

                        // Accept websocket upgrades if the protocol is supported
                        let handle_request = upgrade
                            .use_protocol("flo")
                            .accept()
                            .and_then(move |(client, _headers)| {
                                // We send events to the sink and retrieve updates from the stream (as JSON messages)
                                let (sink, stream) = client.split();

                                // Turn updates into a send_events request
                                let send_events = updates
                                    .map_err(|_|        WebSocketError::NoDataAvailable)
                                    .map(|update|       serde_json::to_string(&update).unwrap())
                                    .map(|update_json|  OwnedMessage::Text(update_json))
                                    .forward(sink)
                                    .and_then(|(_, sink)| {
                                        sink.send(OwnedMessage::Close(None))
                                    });

                                // Turn events from the stream into updates sent to the UI
                                let receive_events = stream
                                    .take_while(|message| Ok(!message.is_close()))
                                    .filter_map(|message| {
                                        match message {
                                            OwnedMessage::Text(event_json)  => Some(event_json),
                                            _                               => None
                                        }
                                    })
                                    .map(|json_string|          serde_json::from_str(&json_string))
                                    .filter_map(|maybe_update|  maybe_update.ok())
                                    .map_err(|_| ())    // TODO: not sure about this
                                    .forward(events)
                                    .and_then(|(_, _)| {
                                        future::ok(())
                                    })
                                    .map_err(|_| WebSocketError::NoDataAvailable); // TODO: want to preserve the original error if any?

                                // Result is a selection of these two futures
                                send_events.map(|_| ()).select(receive_events)
                                    .map(|_| ())
                                    .map_err(|(erm, _next)| erm)
                            });

                        // Spawn our request handler and be done
                        let handle_request = handle_request
                            .map_err(|_err| ())
                            .map(|_| ());
                        tokio_core_handle.spawn(handle_request);
                        Ok(())
                    },

                    None => {
                        // No session at this address: reject the request
                        tokio_core_handle.spawn(upgrade.reject().map_err(|_| ()).map(|_| ()));
                        Ok(())
                    }
                }
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
