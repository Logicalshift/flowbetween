use flo_cocoa_ui::*;

use objc::*;
use objc::runtime::*;

#[no_mangle]
pub unsafe extern fn create_flo_session(window_class: *mut Class, view_class: *mut Class) -> *mut Object {
    // Create the session
    let session: *mut Object    = msg_send!(&**FLO_CONTROL, alloc);
    let session: *mut Object    = msg_send!(session, init);

    // Set the properties
    msg_send!(session, setWindowClass: window_class);
    msg_send!(session, setViewClass: view_class);

    // Retrieve the user interface
    let user_interface = get_session_for_flo_control(&*session)
        .lock().unwrap()
        .create_user_interface();

    session
}
