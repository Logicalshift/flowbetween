use super::*;
use super::super::*;

use binding::*;
use futures::*;
use futures::executor;
use futures::sync::oneshot;

use std::time::*;
use std::thread::*;

struct TestController {
    ui: Binding<Control>
}

impl Controller for TestController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }
}

/// Creates a timeout future
fn timeout(ms: u64) -> oneshot::Receiver<()> {
    let (timeout_send, timeout_recv) = oneshot::channel::<()>();

    spawn(move || {
        sleep(Duration::from_millis(ms));
        timeout_send.send(()).ok();
    });

    timeout_recv
}

#[derive(Clone, PartialEq, Debug)]
enum TestItem {
    Updates(Vec<UiUpdate>),
    Timeout
}

#[test]
fn session_creates_initial_event() {
    // Controller is initially empty
    let controller = TestController { ui: bind(Control::empty()) };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(2000).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    // Fetch the first item from the stream
    let mut next_or_timeout = executor::spawn(next_or_timeout);
    let first_item          = next_or_timeout.wait_stream().unwrap();

    assert!(first_item == Ok(TestItem::Updates(vec![
        UiUpdate::UpdateUi(vec![UiDiff {
            address: vec![],
            new_ui: Control::empty()
        }]),
        
        UiUpdate::UpdateViewModel(vec![])
    ])));
}

#[test]
fn ticks_generate_empty_event() {
    // Controller is initially empty
    let controller = TestController { ui: bind(Control::empty()) };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let mut event_sink  = session.get_input_sink();
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(2000).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    let mut next_or_timeout = executor::spawn(next_or_timeout);

    // Fetch the first item from the stream
    let first_item = next_or_timeout.wait_stream().unwrap();
    assert!(first_item != Ok(TestItem::Timeout));

    // Send a tick
    event_sink.start_send(UiEvent::Tick).unwrap();

    let tick_update = next_or_timeout.wait_stream().unwrap();
    assert!(tick_update == Ok(TestItem::Updates(vec![])));
}

#[test]
fn timeout_after_first_event() {
    // Controller is initially empty
    let controller = TestController { ui: bind(Control::empty()) };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(250).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    let mut next_or_timeout = executor::spawn(next_or_timeout);

    // Fetch the first item from the stream
    let first_item = next_or_timeout.wait_stream().unwrap();
    assert!(first_item != Ok(TestItem::Timeout));

    let should_timeout = next_or_timeout.wait_stream().unwrap();
    assert!(should_timeout == Ok(TestItem::Timeout));
}
