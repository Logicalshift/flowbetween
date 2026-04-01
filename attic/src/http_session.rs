use super::flo_session::*;

use flo_http_ui::*;

impl HttpController for FlowBetweenSession {
    /// Creates a new session
    fn start_new() -> Self {
        let session = FlowBetweenSession::new();

        session
    }
}
