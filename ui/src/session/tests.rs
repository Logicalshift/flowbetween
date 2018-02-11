use super::*;
use super::super::*;

use binding::*;
use futures::*;
use futures::executor;
use futures::sync::oneshot;

use std::time::*;
use std::sync::*;
use std::thread::*;

struct TestController {
    ui: Binding<Control>,
    viewmodel: Option<Arc<DynamicViewModel>>
}

impl Controller for TestController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn get_viewmodel(&self) -> Option<Arc<ViewModel>> { 
        match self.viewmodel {
            Some(ref viewmodel) => Some(viewmodel.clone()),
            None                => None
        }
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
    let controller = TestController { ui: bind(Control::empty()), viewmodel: None };

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
    let controller = TestController { ui: bind(Control::empty()), viewmodel: None };

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

    // Nothing has changed, but the tick should still cause an event
    let tick_update = next_or_timeout.wait_stream().unwrap();
    assert!(tick_update == Ok(TestItem::Updates(vec![])));
}

#[test]
fn timeout_after_first_event() {
    // Controller is initially empty
    let controller = TestController { ui: bind(Control::empty()), viewmodel: None };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(250).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    let mut next_or_timeout = executor::spawn(next_or_timeout);

    // Fetch the first item from the stream
    let first_item = next_or_timeout.wait_stream().unwrap();
    assert!(first_item != Ok(TestItem::Timeout));

    // After the initial event that informs us of the state of the stream, we should block (which will result in the timeout firing here)
    let should_timeout = next_or_timeout.wait_stream().unwrap();
    assert!(should_timeout == Ok(TestItem::Timeout));
}

#[test]
fn ui_update_triggers_update() {
    // Create a UI for us to update later on
    let mut ui = bind(Control::empty());

    // Controller is initially empty
    let controller = TestController { ui: ui.clone(), viewmodel: None };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(1000).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    let mut next_or_timeout = executor::spawn(next_or_timeout);

    // Fetch the first item from the stream
    let first_item = next_or_timeout.wait_stream().unwrap();
    assert!(first_item != Ok(TestItem::Timeout));

    // Update the UI after a short delay (enough time that it'll happen after we start waiting for an update)
    spawn(move || {
        sleep(Duration::from_millis(50));
        ui.set(Control::label().with("Updated"));
    });

    // After the initial event that informs us of the state of the stream, we should block (which will result in the timeout firing here)
    let updated_ui = next_or_timeout.wait_stream().unwrap();
    assert!(updated_ui != Ok(TestItem::Timeout));
    assert!(updated_ui == Ok(TestItem::Updates(vec![
        UiUpdate::UpdateUi(vec![UiDiff {
            address: vec![],
            new_ui: Control::label().with("Updated")
        }])
    ])));
}

/*

-- TODO: viewmodel changes should also result in the update stream returning
-- Viewmodels don't implement Changeable yet which makes this hard to achieve in practice
-- This is not a huge issue because any events will trigger an update, so viewmodel
   changes caused by an event will always be picked up (it's just 'out of band'
   changes that will be missed)

#[test]
fn viewmodel_update_triggers_update() {
    // Create a viewmodel for us to update later on
    let viewmodel = Arc::new(DynamicViewModel::new());

    viewmodel.set_property("Test", PropertyValue::Int(0));

    // Controller is initially empty
    let controller = TestController { ui: bind(Control::empty()), viewmodel: Some(viewmodel.clone()) };

    // Start a UI session for this controller
    let session = UiSession::new(controller);

    // Get an update stream for it and attach a timeout
    let update_stream   = session.get_updates();
    let next_or_timeout = update_stream.map(|updates| TestItem::Updates(updates)).select(timeout(1000).into_stream().map(|_| TestItem::Timeout).map_err(|_| ()));

    let mut next_or_timeout = executor::spawn(next_or_timeout);

    // Fetch the first item from the stream
    let first_item = next_or_timeout.wait_stream().unwrap();
    assert!(first_item != Ok(TestItem::Timeout));

    // Update the viewmodel after a short delay
    spawn(move || {
        sleep(Duration::from_millis(50));
        viewmodel.set_property("Test", PropertyValue::Int(1));
    });

    // After the initial event that informs us of the state of the stream, we should block (which will result in the timeout firing here)
    let updated_ui = next_or_timeout.wait_stream().unwrap();
    assert!(updated_ui != Ok(TestItem::Timeout));
    assert!(updated_ui == Ok(TestItem::Updates(vec![
        UiUpdate::UpdateViewModel(vec![
            ViewModelUpdate::new(vec![], vec![("Test".to_string(), PropertyValue::Int(1))])
        ])
    ])));
}
*/

// TODO: also check we trigger an update if a canvas that's in the UI changes
