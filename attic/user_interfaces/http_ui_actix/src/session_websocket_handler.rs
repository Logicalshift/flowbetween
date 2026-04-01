use super::actix_session::*;

use flo_http_ui::*;
use flo_stream::*;
use flo_ui::*;

use actix::*;
use actix_web::*;
use actix_web::web;
use actix_web_actors::ws;
use futures::*;
use futures::future;
use futures::future::{BoxFuture, LocalBoxFuture};
use futures::channel::oneshot;
use serde_json;

use std::mem;
use std::sync::*;

///
/// Struct used to represent a websocket session
///
struct FloWsSession<Session: ActixSession+'static> {
    /// The session that belongs to this websocket
    session: Arc<Mutex<HttpSession<Session::CoreUi>>>,

    /// The event sink for this session
    event_sink: BoxFuture<'static, WeakPublisher<Vec<Event>>>
}

impl<Session: ActixSession+'static> FloWsSession<Session> {
    ///
    /// Creates a new websocket session
    ///
    pub fn new(session: Arc<Mutex<HttpSession<Session::CoreUi>>>) -> FloWsSession<Session> {
        let event_sink = future::ready(session.lock().unwrap().http_ui().get_input_sink());

        FloWsSession {
            session:    session,
            event_sink: Box::pin(event_sink)
        }
    }

    ///
    /// Starts sending updates to this actor (once a context is available)
    ///
    pub fn start_sending_updates(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // Retrieve the stream of updates we need to send to the websocket
        let update_stream = self.session.lock().unwrap().http_ui().get_updates();
        let update_stream = fut::wrap_stream::<_, Self>(update_stream);

        // Updates are sent to the websocket
        let update_stream = update_stream
            .map(|update, _actor, _ctx| update.map(|update| serde_json::to_string(&update).unwrap()))
            .map(|update, _actor, ctx| update.map(|update| ctx.text(update)));

        // Spawn the updates on the context
        ctx.spawn(update_stream.finish());
    }
}

impl<Session: ActixSession+'static> Actor for FloWsSession<Session> {
    type Context = ws::WebsocketContext<Self>;
}

impl<Session: ActixSession+'static> StreamHandler<Result<ws::Message, ws::ProtocolError>> for FloWsSession<Session> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Text messages are decoded as arrays of HTTP events and sent to the event sink
        match msg {
            Ok(ws::Message::Text(message)) => {
                // Parse the JSON message
                let json = serde_json::from_str::<Vec<Event>>(&message);

                if let Ok(request) = json {
                    // Create a one-shot future for when the event sink is available again
                    let (send_sink, next_sink)  = oneshot::channel();
                    let mut next_sink: BoxFuture<'static, WeakPublisher<Vec<Event>>> = Box::pin(next_sink.map(|result| result.unwrap()));
                    mem::swap(&mut self.event_sink, &mut next_sink);

                    // Spawn the future in this actor
                    ctx.spawn(fut::wrap_future(async move {
                        // Send to the sink
                        let mut event_sink = next_sink.await;
                        event_sink.publish(request).await;
                        send_sink.send(event_sink).ok();
                    }));
                }
            },

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),

            Err(_protocol_error) => { }
            _ => (),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_sending_updates(ctx);
    }
}

///
/// Creates a handler for requests that should spawn a websocket for a session
///
pub fn session_websocket_handler<Session: 'static+ActixSession>(req: HttpRequest, payload: web::Payload) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    // The tail indicates the session ID
    let tail = req.match_info().get("tail").map(|s| String::from(s));

    if let Some(tail) = tail {
        // Strip any preceeding '/'
        let session_id = if tail.chars().nth(0) == Some('/') {
            tail[1..].to_string()
        } else {
            tail.to_string()
        };

        // Look up the session
        let session_state   = req.app_data::<Arc<Session>>().cloned().expect("Flowbetween session state");
        let session         = session_state.get_session(&session_id).clone();

        Box::pin(async move {
            if let Some(session) = session {
                // Start a new websocket for this session
                let session = FloWsSession::<Session>::new(session);

                // Need to perform the handshake manually due to the need to set up the sending stream (actix's model assumes a strict request/response format which is not what we do)
                let response = ws::handshake(&req).map_err(|e| Error::from(e));
                let response = response.map(move |mut response| {
                    // Create the stream
                    let stream = payload;

                    // Apply to the context
                    let ctx = ws::WebsocketContext::create(session, stream);

                    // Generate the response
                    response.streaming(ctx)
                });

                // Generate the websocket response
                response
            } else {
                // Session not found
                Ok(HttpResponse::NotFound().body("Not found"))
            }
        })
    } else {
        // Handler not properly installed, probably
        Box::pin(future::ok(HttpResponse::NotFound().body("Not found")))
    }
}
