use super::core::*;
use super::state::*;
use super::update::*;
use super::super::controller::*;

use desync::*;
use futures::*;

use std::mem;
use std::sync::*;

///
/// Core data for an update stream
/// 
struct UpdateStreamCore {
    /// The controller that will be used to update the state
    controller: Arc<Controller>,

    /// The state of the UI last time an update was generated for the update stream
    state: UiSessionState,

    /// The ID of the last update that was generated
    last_update_id: u64
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
    pub fn new(core: Arc<Desync<UiSessionCore>>, controller: Arc<Controller>) -> UiUpdateStream {
        UiUpdateStream {
            session_core:   core,
            stream_core:    Arc::new(Desync::new(UpdateStreamCore::new(controller))),
            pending:        Arc::new(Mutex::new(None))
        }
    }
}

impl UpdateStreamCore {
    ///
    /// Creates a new update stream core
    /// 
    pub fn new(controller: Arc<Controller>) -> UpdateStreamCore {
        UpdateStreamCore {
            controller:     controller,
            state:          UiSessionState::new(),
            last_update_id: 0
        }
    }
}

impl Stream for UiUpdateStream {
    type Item   = Vec<UiUpdate>;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Vec<UiUpdate>>, Self::Error> {
        // Try to read the pending update, if there is one
        let mut pending         = self.pending.lock().unwrap();
        let mut pending_result  = None;

        mem::swap(&mut pending_result, &mut *pending);
        
        // Result is OK if we found a pending update
        if let Some(pending) = pending_result {
            // There is a pending update
            Ok(Async::Ready(Some(pending)))
        } else {
            // No update available yet. We need to register with the core to trigger one
            let task            = task::current();
            let pending         = Arc::clone(&self.pending);
            let session_core    = Arc::clone(&self.session_core);
            let stream_core     = Arc::clone(&self.stream_core);

            session_core.async(move |session_core| {
                stream_core.sync(move |stream_core| {
                    if pending.lock().unwrap().is_some() {
                        // If there's now a pending update, then signal the task to return via the stream
                        task.notify();
                    } else if session_core.last_update_id() != stream_core.last_update_id {
                        // If the core has a newer update than we do then start generating a new pending update
                        unimplemented!()
                    } else {
                        // Otherwise, ask the core to notify us when an update is available
                        unimplemented!()
                    }
                });
            });

            // Not ready yet
            Ok(Async::NotReady)
        }
    }
}
