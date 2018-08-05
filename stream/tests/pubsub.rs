extern crate futures;
extern crate flo_stream;

use flo_stream::*;

use futures::*;
use futures::executor;
use futures::executor::{Notify, NotifyHandle};

#[test]
fn can_receive_on_one_subscriber() {
    let mut publisher   = Publisher::new(10);
    let subscriber      = publisher.subscribe();

    let mut publisher   = executor::spawn(publisher);
    let mut subscriber  = executor::spawn(subscriber);

    publisher.wait_send(1).unwrap();
    publisher.wait_send(2).unwrap();
    publisher.wait_send(3).unwrap();

    assert!(subscriber.wait_stream() == Some(Ok(1)));
    assert!(subscriber.wait_stream() == Some(Ok(2)));
    assert!(subscriber.wait_stream() == Some(Ok(3)));
}

#[test]
fn subscriber_closes_when_publisher_closes() {
    let mut closed_subscriber = {
        let mut publisher   = Publisher::new(10);
        let subscriber      = publisher.subscribe();

        let mut publisher   = executor::spawn(publisher);
        let mut subscriber  = executor::spawn(subscriber);

        publisher.wait_send(1).unwrap();

        assert!(subscriber.wait_stream() == Some(Ok(1)));
        subscriber
    };

    assert!(closed_subscriber.wait_stream() == None);
}

#[test]
fn can_read_on_multiple_subscribers() {
    let mut publisher   = Publisher::new(10);
    let subscriber1     = publisher.subscribe();
    let subscriber2     = publisher.subscribe();

    let mut publisher   = executor::spawn(publisher);
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
fn will_skip_messages_sent_before_subscription() {
    let publisher       = Publisher::new(10);
    let mut publisher   = executor::spawn(publisher);

    publisher.wait_send(1).unwrap();
    let subscriber1     = publisher.get_mut().subscribe();
    publisher.wait_send(2).unwrap();
    let subscriber2     = publisher.get_mut().subscribe();
    publisher.wait_send(3).unwrap();

    let mut subscriber1 = executor::spawn(subscriber1);
    let mut subscriber2 = executor::spawn(subscriber2);

    assert!(subscriber1.wait_stream() == Some(Ok(2)));
    assert!(subscriber1.wait_stream() == Some(Ok(3)));

    assert!(subscriber2.wait_stream() == Some(Ok(3)));
}

#[derive(Clone)]
struct NotifyNothing;

impl Notify for NotifyNothing {
    fn notify(&self, _: usize) { }
}

#[test]
fn will_block_if_subscribers_are_full() {
    let mut publisher   = Publisher::new(10);
    let subscriber      = publisher.subscribe();

    let mut publisher   = executor::spawn(publisher);
    let mut subscriber  = executor::spawn(subscriber);

    let mut send_count  = 0;
    while send_count < 100 {
        match publisher.start_send_notify(send_count, &NotifyHandle::from(&NotifyNothing), 0) {
            Ok(AsyncSink::NotReady(_)) => {
                break;
            },

            _ => { }
        }

        send_count += 1;
    }

    assert!(send_count == 10);

    for a in 0..10 {
        assert!(subscriber.wait_stream() == Some(Ok(a)));
    }
}
