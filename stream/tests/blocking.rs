extern crate futures;
extern crate flo_stream;

use flo_stream::*;

use futures::*;
use futures::executor;
use futures::executor::{Notify, NotifyHandle};

use std::thread;
use std::sync::*;
use std::time::Duration;

#[derive(Clone)]
struct NotifyNothing;

impl Notify for NotifyNothing {
    fn notify(&self, _: usize) { }
}

#[test]
pub fn blocks_if_there_are_no_subscribers() {
    let publisher       = BlockingPublisher::new(2, 10);
    let mut publisher   = executor::spawn(publisher);

    assert!(publisher.start_send_notify(1, &NotifyHandle::from(&NotifyNothing), 0) == Ok(AsyncSink::NotReady(1)));
}

#[test]
pub fn blocks_if_there_are_insufficient_subscribers() {
    let publisher       = BlockingPublisher::new(2, 10);
    let mut publisher   = executor::spawn(publisher);
    let _subscriber     = publisher.subscribe();

    assert!(publisher.start_send_notify(1, &NotifyHandle::from(&NotifyNothing), 0) == Ok(AsyncSink::NotReady(1)));
}

#[test]
pub fn unblocks_if_there_are_sufficient_subscribers() {
    let publisher       = BlockingPublisher::new(2, 10);
    let mut publisher   = executor::spawn(publisher);
    let _subscriber1    = publisher.subscribe();
    let _subscriber2    = publisher.subscribe();

    assert!(publisher.start_send_notify(1, &NotifyHandle::from(&NotifyNothing), 0) == Ok(AsyncSink::Ready));
}

#[test]
pub fn read_from_subscribers() {
    let publisher       = BlockingPublisher::new(2, 10);
    let mut publisher   = executor::spawn(publisher);
    let subscriber1     = publisher.subscribe();
    let subscriber2     = publisher.subscribe();

    let mut subscriber1 = executor::spawn(subscriber1);
    let mut subscriber2 = executor::spawn(subscriber2);

    publisher.wait_send(1).unwrap();
    publisher.wait_send(2).unwrap();
    publisher.wait_send(3).unwrap();

    assert!(subscriber1.wait_stream() == Some(Ok(1)));
    assert!(subscriber1.wait_stream() == Some(Ok(2)));
    assert!(subscriber1.wait_stream() == Some(Ok(3)));

    assert!(subscriber2.wait_stream() == Some(Ok(1)));
    assert!(subscriber2.wait_stream() == Some(Ok(2)));
    assert!(subscriber2.wait_stream() == Some(Ok(3)));
}

#[test]
pub fn read_from_thread() {
    let subscriber = {
        // Create a shared publisher
        let publisher = BlockingPublisher::new(1, 1);
        let publisher = executor::spawn(publisher);
        let publisher = Arc::new(Mutex::new(publisher));

        // Create a thread to publish some values
        let thread_publisher = publisher.clone();
        thread::spawn(move || {
            let wait_for_subscribers = thread_publisher.lock().unwrap().get_mut().when_ready();
            executor::spawn(wait_for_subscribers).wait_future().unwrap();

            thread_publisher.lock().unwrap().wait_send(1).unwrap();
            thread_publisher.lock().unwrap().wait_send(2).unwrap();
            thread_publisher.lock().unwrap().wait_send(3).unwrap();
        });

        // Pause for a bit to let the thread get ahead of us
        thread::sleep(Duration::from_millis(20));

        // Subscribe to the thread (which should now wake up)
        let subscriber = publisher.lock().unwrap().subscribe();
        subscriber
    };

    let mut subscriber = executor::spawn(subscriber);

    // Should receive the values from the thread
    assert!(subscriber.wait_stream() == Some(Ok(1)));
    assert!(subscriber.wait_stream() == Some(Ok(2)));
    assert!(subscriber.wait_stream() == Some(Ok(3)));

    // As we don't retain the publisher, the thread is its only owner. When it finishes, the stream should close.
    assert!(subscriber.wait_stream() == None);
}

#[test]
pub fn read_from_thread_late_start() {
    let subscriber = {
        // Create a shared publisher
        let publisher = BlockingPublisher::new(1, 1);
        let publisher = executor::spawn(publisher);
        let publisher = Arc::new(Mutex::new(publisher));

        // Create a thread to publish some values
        let thread_publisher = publisher.clone();
        thread::spawn(move || {
            // Wait for the subscriber to be created
            thread::sleep(Duration::from_millis(20));

            let wait_for_subscribers = thread_publisher.lock().unwrap().get_mut().when_ready();
            executor::spawn(wait_for_subscribers).wait_future().unwrap();

            thread_publisher.lock().unwrap().wait_send(1).unwrap();
            thread_publisher.lock().unwrap().wait_send(2).unwrap();
            thread_publisher.lock().unwrap().wait_send(3).unwrap();
        });

        // Subscribe to the thread (which should now wake up)
        let subscriber = publisher.lock().unwrap().subscribe();
        subscriber
    };

    let mut subscriber = executor::spawn(subscriber);

    // Should receive the values from the thread
    assert!(subscriber.wait_stream() == Some(Ok(1)));
    assert!(subscriber.wait_stream() == Some(Ok(2)));
    assert!(subscriber.wait_stream() == Some(Ok(3)));

    // As we don't retain the publisher, the thread is its only owner. When it finishes, the stream should close.
    assert!(subscriber.wait_stream() == None);
}
