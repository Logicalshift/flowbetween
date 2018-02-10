use super::core::*;
use super::state::*;
use super::update::*;

use desync::*;
use futures::*;

use std::sync::*;

///
/// Core data for an update stream
/// 
struct UpdateStreamCore {
    /// The state of the UI last time an update was generated for the update stream
    state: UiSessionState
}

///
/// Stream that can be used to retrieve the most recent set of UI updates from
/// the core. It's possible to retrieve empty updates in the event the core processed
/// events that produced no changes (ie, sending an event to the sink will cause this
/// stream to eventually return at least one update set)
/// 
/// Every update stream begins with an update that sets the initial state of the
/// UI.
/// 
pub struct UiUpdateStream {
    /// The session core
    session_core: Arc<Desync<UiSessionCore>>,

    /// The stream core
    stream_core: Arc<Desync<UpdateStreamCore>>,

    /// Update that was generated for the last poll and is ready to go
    pending: Arc<Mutex<Option<Vec<UiUpdate>>>>
}

impl UiUpdateStream {
    ///
    /// Creates a new UI update stream
    /// 
    pub fn new(core: Arc<Desync<UiSessionCore>>) -> UiUpdateStream {
        UiUpdateStream {
            session_core:   core,
            stream_core:    Arc::new(Desync::new(UpdateStreamCore::new())),
            pending:        Arc::new(Mutex::new(None))
        }
    }
}

impl UpdateStreamCore {
    ///
    /// Creates a new update stream core
    /// 
    pub fn new() -> UpdateStreamCore {
        UpdateStreamCore {
            state: UiSessionState::new()
        }
    }
}

impl Stream for UiUpdateStream {
    type Item   = Vec<UiUpdate>;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Vec<UiUpdate>>, Self::Error> {
        unimplemented!()
    }
}
