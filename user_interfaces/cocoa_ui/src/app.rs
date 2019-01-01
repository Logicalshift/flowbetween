use super::session::*;

use flo_cocoa_pipe::*;

use futures::*;
use futures::stream;
use objc::declare::*;
use objc::runtime::{Object, Class, Sel};
use cocoa::base::*;
use cocoa::appkit::*;
use cocoa::foundation::*;

use std::sync::*;
use std::thread;
use std::collections::HashMap;

lazy_static! {
    /// The FloControl objective-C class
    pub static ref FLO_CONTROL: &'static Class = declare_flo_control_class();

    /// The active sessions
    pub static ref FLO_SESSIONS: Mutex<HashMap<usize, Arc<Mutex<CocoaSession>>>> = Mutex::new(HashMap::new());

    /// The ID to use for the next session
    pub static ref NEXT_SESSION_ID: Mutex<usize> = Mutex::new(0);
}

///
/// Runs a Cocoa application thread
/// 
/// The input is the actions the application should take. The output is the events generated from the UI
/// this forms. The application is ultimately run on a separate 
///
pub fn run_cocoa_application<ActionStream>(actions: ActionStream) -> impl Send+Stream<Item=AppEvent, Error=()>
where ActionStream: Send+Stream<Item=AppAction, Error=()> {
    // Start a thread to act as the main cocoa thread
    thread::spawn(|| {
        run_cocoa_thread();
    });

    stream::empty()
}

// Creating a new instance of the class initializes the session
extern fn init_flo_control(this: &Object, _cmd: Sel) -> *mut Object {
    unsafe {
        let this: *mut Object = msg_send!(super(this, class!(NSObject)), init);

        if this != nil {
            // Assign a session ID
            let mut next_session_id = NEXT_SESSION_ID.lock().unwrap();
            let this_session_id     = *next_session_id;
            (*next_session_id)      += 1;

            (*this).set_ivar("_sessionId", this_session_id);

            // Create the session itself
            let mut sessions        = FLO_SESSIONS.lock().unwrap();
            let new_session         = CocoaSession::new();
            sessions.insert(this_session_id, Arc::new(Mutex::new(new_session)));
        }

        this
    }
}

///
/// Declares the FloControl class to objective-C
/// 
/// The FloControl class is used to dispatch messages from the action stream to objective C. It wraps a Cocoa
/// session.
///
fn declare_flo_control_class() -> &'static Class {
    // Create the class declaration
    let mut flo_control = ClassDecl::new("FloControl", class!(NSObject)).unwrap();

    unsafe {
        // Class contains a session ID we can use to look up the main session
        flo_control.add_ivar::<usize>("_sessionId");

        // Register the init function
        flo_control.add_method(sel!(init), init_flo_control as extern fn(&Object, Sel) -> *mut Object);
    }

    // Seal and register it
    flo_control.register()
}

///
/// Actually runs the main Cocoa thread
///
fn run_cocoa_thread() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        // Set up the application
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        // msg_send![class!(NSObject), performSelectorOnMainThread: sel!(foo) withObject: nil waitUntilDone: YES];

        // Actually run the applicaiton
        app.run();
    }
}