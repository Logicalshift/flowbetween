use super::session::*;

use objc::rc::*;
use objc::declare::*;
use objc::runtime::{Object, Class, Sel};
use cocoa::base::*;

use std::sync::*;
use std::collections::HashMap;

lazy_static! {
    /// The FloControl objective-C class
    pub static ref FLO_CONTROL: &'static Class = declare_flo_control_class();

    /// The active sessions
    static ref FLO_SESSIONS: Mutex<HashMap<usize, Arc<Mutex<CocoaSession>>>> = Mutex::new(HashMap::new());

    /// The ID to use for the next session
    static ref NEXT_SESSION_ID: Mutex<usize> = Mutex::new(0);
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
                    let new_session         = CocoaSession::new(&this_ptr, this_session_id);
                    sessions.insert(this_session_id, Arc::new(Mutex::new(new_session)));
                }

                this
            }
        }

        /// Sets the window class for a FloControl object
        extern fn set_window_class(this: &mut Object, _cmd: Sel, new_window_class: *mut Class) {
            unsafe {
                let _: id = msg_send!(new_window_class, retain);
                this.set_ivar("_windowClass", new_window_class);
            }
        }

        /// Sets the view class for a FloControl object
        extern fn set_view_class(this: &mut Object, _cmd: Sel, new_view_class: *mut Class) {
            unsafe {
                let _: id = msg_send!(new_view_class, retain);
                this.set_ivar("_viewClass", new_view_class);
            }
        }

        /// Sets the viewmodel class for a FloControl object
        extern fn set_view_model_class(this: &mut Object, _cmd: Sel, new_view_model_class: *mut Class) {
            unsafe {
                let _: id = msg_send!(new_view_model_class, retain);
                this.set_ivar("_viewModelClass", new_view_model_class);
            }
        }

        /// Drains the action stream when it's ready
        extern fn action_stream_ready(this: &mut Object, _cmd: Sel) {
            unsafe {
                let session_id  = this.get_ivar("_sessionId");
                let session     = FLO_SESSIONS.lock().unwrap().get(&session_id).cloned();

                if let Some(session) = session {
                    let mut session = session.lock().unwrap();

                    session.drain_action_stream();
                }
            }
        }

        /// Tidies up the flo_control session once it has finished
        extern fn dealloc_flo_control(this: &mut Object, _cmd: Sel) {
            unsafe {
                println!("Deallocating FloControl (trying to end session...)");

                // Remove the session from the session hash
                let session_id = this.get_ivar("_sessionId");
                FLO_SESSIONS.lock().unwrap().remove(&session_id);
            }
        }

        /// Retrieves the session ID for this object
        extern fn get_session_id(this: &mut Object, _cmd: Sel) -> u64 {
            unsafe {
                // Remove the session from the session hash
                let session_id = this.get_ivar::<usize>("_sessionId");
                *session_id as u64
            }
        }

        /// Sends a tick to the session
        extern fn tick(this: &mut Object, _cmd: Sel) {
            unsafe {
                // Send a tick to the session
                let session_id  = this.get_ivar("_sessionId");
                let session     = FLO_SESSIONS.lock().unwrap().get(&session_id).cloned();

                if let Some(session) = session {
                    let mut session = session.lock().unwrap();

                    session.tick();
                }
            }
        }

        // Class contains a session ID we can use to look up the main session
        flo_control.add_ivar::<usize>("_sessionId");

        // We delegate messages to the window and view classes
        flo_control.add_ivar::<*mut Class>("_windowClass");
        flo_control.add_ivar::<*mut Class>("_viewClass");
        flo_control.add_ivar::<*mut Class>("_viewModelClass");

        // Register the init function
        flo_control.add_method(sel!(init), init_flo_control as extern fn(&Object, Sel) -> *mut Object);
        flo_control.add_method(sel!(dealloc), dealloc_flo_control as extern fn(&mut Object, Sel));
        flo_control.add_method(sel!(setWindowClass:), set_window_class as extern fn(&mut Object, Sel, *mut Class));
        flo_control.add_method(sel!(setViewClass:), set_view_class as extern fn(&mut Object, Sel, *mut Class));
        flo_control.add_method(sel!(setViewModelClass:), set_view_model_class as extern fn(&mut Object, Sel, *mut Class));
        flo_control.add_method(sel!(actionStreamReady), action_stream_ready as extern fn(&mut Object, Sel));
        flo_control.add_method(sel!(tick), tick as extern fn(&mut Object, Sel));
        flo_control.add_method(sel!(sessionId), get_session_id as extern fn(&mut Object, Sel) -> u64);
    }

    // Seal and register it
    flo_control.register()
}

///
/// Retrieves the session for a FloControl objective-C object
///
pub unsafe fn get_session_for_flo_control(flo_control: &Object) -> Arc<Mutex<CocoaSession>> {
    let session_id = flo_control.get_ivar("_sessionId");
    FLO_SESSIONS.lock().unwrap()
        .get(&session_id)
        .cloned()
        .unwrap()
}

///
/// Retrieves the session for a FloControl objective-C object
///
pub fn get_cocoa_session_with_id(session_id: usize) -> Option<Arc<Mutex<CocoaSession>>> {
    FLO_SESSIONS.lock().unwrap()
        .get(&session_id)
        .cloned()
}
