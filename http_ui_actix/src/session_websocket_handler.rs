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

    ///
    /// Starts sending updates to this actor
    /// 
    pub fn start_sending_updates(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // Retrieve the stream of updates we need to send to the websocket
        let update_stream = self.session.lock().unwrap().http_ui().get_updates();
        let update_stream = fut::wrap_stream::<_, Self>(update_stream);

        // Updates are sent to the websocket
        let update_stream = update_stream
            .map(|update, _actor, _ctx| serde_json::to_string(&update).unwrap())
            .map(|update, _actor, ctx| ctx.text(update));
        
        // Spawn the updates on the context
        ctx.spawn(update_stream.finish());
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
