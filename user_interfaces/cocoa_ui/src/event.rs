use flo_stream::*;
use flo_cocoa_pipe::*;

use objc::*;
use objc::rc::*;
use objc::declare::*;
use objc::runtime::*;

use futures::executor;
use futures::executor::Spawn;

use std::sync::*;
use std::collections::HashMap;

lazy_static! {
    /// The events class
    pub static ref FLO_EVENTS_CLASS: &'static Class = declare_flo_events_class();

    /// Used to look up the FloEvents object associated with an instance of the FloEvents class
    static ref FLO_EVENTS_STORE: Mutex<HashMap<usize, Arc<Mutex<FloEvents>>>> = Mutex::new(HashMap::new());

    /// The next ID to assign to a FloEvents object
    static ref NEXT_FLO_EVENTS_ID: Mutex<usize> = Mutex::new(0);
}

///
/// Event target for sending events from the objective-C side to the rust side
///
pub struct FloEvents {
    events_publisher: Spawn<Publisher<Vec<AppEvent>>>
}

impl FloEvents {
    ///
    /// Creates a new FloEvents
    ///
    pub fn init(publisher: Publisher<Vec<AppEvent>>) -> FloEvents {
        FloEvents {
            events_publisher: executor::spawn(publisher)
        }
    }

    ///
    /// Creates the objective-C object from a FloEvents reference
    ///
    fn object_from_events(events: Arc<Mutex<FloEvents>>) -> StrongPtr {
        unsafe {
            // Assign an ID to this events object
            let events_id = {
                let mut next_flo_events_id  = NEXT_FLO_EVENTS_ID.lock().unwrap();
                let events_id               = *next_flo_events_id;
                *next_flo_events_id         += 1;

                events_id
            };

            // Store the events away
            FLO_EVENTS_STORE.lock().unwrap().insert(events_id, events);

            // Allocate the object
            let flo_events_object: *mut Object = msg_send!(*FLO_EVENTS_CLASS, alloc);
            let flo_events_object: *mut Object = msg_send!(flo_events_object, init);

            // Set it up
            (*flo_events_object).set_ivar("_eventsId", events_id);

            StrongPtr::new(flo_events_object)
        }
    }

    ///
    /// Creates a new FloEvents object for objective-C
    ///
    pub fn create_object(publisher: Publisher<Vec<AppEvent>>) -> StrongPtr {
        let events = FloEvents::init(publisher);
        Self::object_from_events(Arc::new(Mutex::new(events)))
    }
}

///
/// Declares the FloEvents objective C class
///
pub fn declare_flo_events_class() -> &'static Class {
    // Create the class declaration
    let mut flo_events = ClassDecl::new("FloEvents", class!(NSObject)).unwrap();

    // Add the event ID ivar
    flo_events.add_ivar::<usize>("_eventsId");

    // Sends an event to the events object
    unsafe fn send_event(flo_events: *mut Object, event: AppEvent) {
        let events_id   = (*flo_events).get_ivar::<usize>("_eventsId");
        let flo_events  = FLO_EVENTS_STORE.lock().unwrap().get(events_id).cloned();

        flo_events.map(|flo_events| flo_events.lock().unwrap().events_publisher.wait_send(vec![event]).ok());
    }

    // Finalize the class
    flo_events.register()
}
