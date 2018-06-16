use flo_http_ui::*;
use flo_ui::session::*;

use actix::*;
use actix_web::*;

use std::sync::*;

///
/// Struct used to represent a websocket session
/// 
struct FloWsSession<CoreUi> {
    /// The session that belongs to this websocket
    session: Arc<Mutex<HttpSession<CoreUi>>>
}

impl<CoreUi: 'static> Actor for FloWsSession<CoreUi> {
    type Context = ws::WebsocketContext<Self>;
}

impl<CoreUi: CoreUserInterface+Send+Sync+'static> StreamHandler<ws::Message, ws::ProtocolError> for FloWsSession<CoreUi> {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            _ => (),
        }
    }
}
