#[macro_use] extern crate serde_derive;

mod flo_session;
mod logo_controller;
use self::flo_session::*;

use flo_ui::session::*;
use flo_cocoa_ui::*;
use flo_cocoa_pipe::*;

use objc::*;
use objc::runtime::*;
use futures::executor;

use std::thread;

#[no_mangle]
pub unsafe extern fn create_flo_session(window_class: *mut Class, view_class: *mut Class, view_model_class: *mut Class) -> *mut Object {
    // Create the session
    let session: *mut Object    = msg_send!(&**FLO_CONTROL, alloc);
    let session: *mut Object    = msg_send!(session, init);

    // Set the properties
    let _: () = msg_send!(session, setWindowClass: window_class);
    let _: () = msg_send!(session, setViewClass: view_class);
    let _: () = msg_send!(session, setViewModelClass: view_model_class);

    // Retrieve the user interface
    let user_interface = get_session_for_flo_control(&*session)
        .lock().unwrap()
        .create_user_interface();

    // Spawn a thread to send the updates back and forth
    thread::Builder::new()
        .name("FlowBetween session".to_string())
        .spawn(move || {
            // Create a new flo session
            let flo                 = FlowBetweenSession::new();
            let flo_session         = UiSession::new(flo);

            // Pipe into the UI
            let update_future       = pipe_ui_updates(&flo_session, &user_interface);

            // Wait for the future to complete
            let mut update_future   = executor::spawn(update_future);

            update_future.wait_future().ok();
            println!("Session finished");
        })
        .unwrap();

    let _: *mut Object = msg_send!(session, autorelease);
    session
}
