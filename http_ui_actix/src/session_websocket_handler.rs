use flo_http_ui::*;
use flo_ui::*;
use flo_ui::session::*;

use actix::*;
use actix::fut;
use actix_web::*;
use futures::*;
use futures::future;
use futures::sync::oneshot;
use serde_json;

use std::mem;
use std::sync::*;

///
/// Struct used to represent a websocket session
/// 
struct FloWsSession<CoreUi: CoreUserInterface+Send+Sync+'static> {
    /// The session that belongs to this websocket
    session: Arc<Mutex<HttpSession<CoreUi>>>,

    /// The event sink for this session
    event_sink: Box<Future<Item=HttpEventSink, Error=()>>
}

impl<CoreUi: CoreUserInterface+Send+Sync+'static> FloWsSession<CoreUi> {
    ///
    /// Creates a new websocket session
    /// 
    pub fn new(session: Arc<Mutex<HttpSession<CoreUi>>>) -> FloWsSession<CoreUi> {
        let event_sink = future::ok(session.lock().unwrap().http_ui().get_input_sink());

        FloWsSession {
            session:    session,
            event_sink: Box::new(event_sink)
        }
    }
}

impl<CoreUi: CoreUserInterface+Send+Sync+'static> Actor for FloWsSession<CoreUi> {
    type Context = ws::WebsocketContext<Self>;
}

impl<CoreUi: CoreUserInterface+Send+Sync+'static> StreamHandler<ws::Message, ws::ProtocolError> for FloWsSession<CoreUi> {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Text(message) => {
                // Parse the JSON message
                let json = serde_json::from_str::<Vec<Event>>(&message);

                if let Ok(request) = json {
                    // Create a one-shot future for when the event sink is available again
                    let (send_sink, next_sink)  = oneshot::channel();
                    let mut next_sink: Box<Future<Item=HttpEventSink, Error=()>> = Box::new(next_sink.map_err(|_| ()));
                    mem::swap(&mut self.event_sink, &mut next_sink);
                    
                    // Send to the sink
                    let send_future = next_sink
                        .and_then(|event_sink| event_sink.send(request))
                        .map(move |event_sink| { send_sink.send(event_sink).ok(); });
                    
                    // Spawn the future in this actor
                    ctx.spawn(fut::wrap_future(send_future));
                }
            },

            ws::Message::Ping(msg) => ctx.pong(&msg),
            _ => (),
        }
    }
}
