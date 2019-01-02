use super::session::*;

use flo_cocoa_pipe::*;

use futures::*;
use futures::stream;
use objc::rc::*;
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
        /// Creating a new instance of the class initializes the session
        extern fn init_flo_control(this: &Object, _cmd: Sel) -> *mut Object {
            unsafe {
                let this: *mut Object = msg_send!(super(this, class!(NSObject)), init);

                if this != nil {
                    // Assign a session ID
                    let mut next_session_id = NEXT_SESSION_ID.lock().unwrap();
                    let this_session_id     = *next_session_id;
                    (*next_session_id)      += 1;

                    (*this).set_ivar("_sessionId", this_session_id);

                    // Retain a copy of this for use with the session
                    let this_ptr = StrongPtr::retain(this);

                    // Create the session itself
                    let mut sessions        = FLO_SESSIONS.lock().unwrap();
                    let new_session         = CocoaSession::new(&this_ptr);
                    sessions.insert(this_session_id, Arc::new(Mutex::new(new_session)));
                }

                this
            }
        }

        /// Sets the window class for a FloControl object
        extern fn set_window_class(this: &mut Object, _cmd: Sel, new_window_class: *mut Class) {
            unsafe {
                msg_send!(new_window_class, retain);
                this.set_ivar("_windowClass", new_window_class);
            }
        }

        /// Sets the view class for a FloControl object
        extern fn set_view_class(this: &mut Object, _cmd: Sel, new_view_class: *mut Class) {
            unsafe {
                msg_send!(new_view_class, retain);
                this.set_ivar("_viewClass", new_view_class);
            }
        }

        // Class contains a session ID we can use to look up the main session
        flo_control.add_ivar::<usize>("_sessionId");

        // We delegate messages to the window and view classes
        flo_control.add_ivar::<*mut Class>("_windowClass");
        flo_control.add_ivar::<*mut Class>("_viewClass");

        // Register the init function
        flo_control.add_method(sel!(init), init_flo_control as extern fn(&Object, Sel) -> *mut Object);
        flo_control.add_method(sel!(setWindowClass:), set_window_class as extern fn(&mut Object, Sel, *mut Class));
        flo_control.add_method(sel!(setViewClass:), set_view_class as extern fn(&mut Object, Sel, *mut Class));
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