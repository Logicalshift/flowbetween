use super::focus::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::*;

use futures::prelude::*;
use serde::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Gesture {
    Focus(FocusEvent),
}

///
/// Runs a gesture program, forwarding any focus events originally destined for the current program to the new program
///
pub async fn run_gesture_program(context: &SceneContext) {
    let active_program_id  = context.current_program_id().unwrap();
    let gesture_program_id = SubProgramId::new();

    let mut scene_control = context.send(()).unwrap();
    
    // Run the gesture program
    scene_control.send(SceneControl::start_child_program(gesture_program_id, active_program_id, move |input, context| {
        gesture_program(input, context)
    }, 1)).await.ok();

    // Send messages from the gesture program back to the original program
    scene_control.send(SceneControl::connect(gesture_program_id, active_program_id, StreamId::with_message_type::<Gesture>())).await.ok();

    // Redirect messages intended for the source program to the gesture program
    scene_control.send(SceneControl::connect((), gesture_program_id, StreamId::with_message_type::<FocusEvent>().for_target(active_program_id))).await.ok();
}

///
/// The gesture program takes focus events and converts them to more interesting gesture events (like drags, drops, etc)
///
/// Unrecognised focus events are passed on.
///
/// Create the subprogram when the mouse down event is detected, then forward any focus events here
///
pub async fn gesture_program(input: InputStream<FocusEvent>, context: SceneContext) {
    // We'll forward unhandled focus events back to the main program
    // TODO: think this ends up sending them back to us...
    let mut forward_gesture = context.send(()).ok();

    // Nothing to do if we fail to connect to both the focus and gesture events
    if forward_gesture.is_none() {
        return;
    }

    // Run the main event loop
    let mut input = input;
    while let Some(evt) = input.next().await {
        match evt {
            other => {
                // Other types of events are just forwarded immediately
                if let Some(gesture) = &mut forward_gesture {
                    if gesture.send(Gesture::Focus(other)).await.is_err() {
                        forward_gesture = None;
                    }
                }
            }
        }

        if forward_gesture.is_none() {
            break;
        }
    }
}

impl SceneMessage for Gesture {
    fn message_type_name() -> String { "flowbetween::Gesture".into() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn convert_event_to_gesture() {
        let scene           = Scene::default();
        let parent_program  = SubProgramId::new();
        let test_program    = SubProgramId::new();

        // Add a subprogram that sends focus events to the test program
        scene.add_subprogram(parent_program, move |input, context| async move {
            // The gesture program intercepts focus events destined for this program
            run_gesture_program(&context).await;

            let mut test_program = context.send(test_program).unwrap();

            // Relay focus events to the test program
            let mut input = input;
            while let Some(gesture) = input.next().await {
                let gesture: Gesture = gesture;
                test_program.send(gesture).await.unwrap();
            }
        }, 20);

        // Test is to send the message to the parent program and expect it to get relayed back to the test program
        TestBuilder::new()
            .send_message_to_target(parent_program, FocusEvent::Event(None, DrawEvent::NewFrame))
            .expect_message_matching(Gesture::Focus(FocusEvent::Event(None, DrawEvent::NewFrame)), "Unexpected Gesture")
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    #[test]
    pub fn forward_unknown_focus_event_with_filter() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum TestMessage {
            Focus(FocusEvent),
            Gesture(Gesture)
        }

        impl SceneMessage for TestMessage { }

        let scene           = Scene::default();
        let parent_program  = SubProgramId::new();
        let test_program    = SubProgramId::new();

        // Turn focus and gesture messages into test messages
        // TODO: if we specifiy a filter for focus events, they don't redirect (which perhaps they should do for consistency)
        // scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|evts| evts.map(|evt| TestMessage::Focus(evt)))), (), StreamId::with_message_type::<FocusEvent>()).unwrap();
        scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|evts| evts.map(|evt| TestMessage::Gesture(evt)))), (), StreamId::with_message_type::<Gesture>()).unwrap();

        // Add a subprogram that sends focus events to the test program
        scene.add_subprogram(parent_program, move |input, context| async move {
            // The gesture program intercepts focus events destined for this program
            run_gesture_program(&context).await;

            let mut test_program = context.send(test_program).unwrap();

            // Relay focus events to the test program
            let mut input = input;
            while let Some(test_message) = input.next().await {
                let test_message: TestMessage = test_message;
                test_program.send(test_message).await.unwrap();
            }
        }, 20);

        // Test is to send the message to the parent program and expect it to get relayed back to the test program
        TestBuilder::new()
            .send_message_to_target(parent_program, FocusEvent::Event(None, DrawEvent::NewFrame))
            .expect_message_matching(TestMessage::Gesture(Gesture::Focus(FocusEvent::Event(None, DrawEvent::NewFrame))), "Unexpected Gesture")
            .run_in_scene_with_threads(&scene, test_program, 5);
    }
}