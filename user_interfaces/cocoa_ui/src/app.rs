use flo_cocoa_pipe::*;

use futures::*;
use futures::stream;
use cocoa::base::*;
use cocoa::appkit::*;
use cocoa::foundation::*;

use std::thread;

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