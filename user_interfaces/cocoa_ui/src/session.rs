use flo_cocoa_pipe::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

use objc::rc::*;
use objc::runtime::*;

///
/// Basis class for a Cocoa session
///
pub struct CocoaSession {
    /// Reference to the FloControl we'll relay the stream via
    target_object: WeakPtr,

    /// The stream of actions for this session (or None if we aren't monitoring for actions)
    actions: Option<Spawn<Box<dyn Stream<Item=AppAction, Error=()>+Send>>>
}

impl CocoaSession {
    ///
    /// Creates a new CocoaSession
    ///
    pub fn new(obj: &Object) -> CocoaSession {
        CocoaSession {
            target_object:  obj.weak(),
            actions:        None
        }
    }

    ///
    /// Listens for actions from the specified stream
    ///
    pub fn listen_to<Actions>(&mut self, actions: Actions)
    where Actions: 'static+Send+Stream<Item=AppAction, Error=()> {
        // Spawn the actions stream
        self.actions = Some(executor::spawn(Box::new(actions)));
    }
}
