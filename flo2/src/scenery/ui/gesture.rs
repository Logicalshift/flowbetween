use super::focus::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::*;

use futures::prelude::*;

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
    scene_control.send(SceneControl::connect(gesture_program_id, active_program_id, StreamId::with_message_type::<FocusEvent>())).await.ok();

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
    let mut forward_focus = context.send(()).unwrap();

    // Run the main event loop
    let mut input = input;
    while let Some(evt) = input.next().await {
        match evt {
            other => {
                // Other types of events are just forwarded immediately
                if forward_focus.send(other).await.is_err() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn forward_without_gesture_program() {
        let scene           = Scene::default();
        let parent_program  = SubProgramId::new();
        let test_program    = SubProgramId::new();

        // Add a subprogram that sends focus events to the test program
        // This test verifies that the other tests are working properly, as we don't use the gesture program so events shouldn't be intercepted
        scene.add_subprogram(parent_program, move |input, context| async move {
            let mut test_program = context.send(test_program).unwrap();

            // Relay focus events to the test program
            let mut input = input;
            while let Some(focus_event) = input.next().await {
                let focus_event: FocusEvent = focus_event;
                test_program.send(focus_event).await.unwrap();
            }
        }, 20);

        // Test is to send the message to the parent program and expect it to get relayed back to the test program
        TestBuilder::new()
            .send_message_to_target(parent_program, FocusEvent::Event(None, DrawEvent::NewFrame))
            .expect_message_matching(FocusEvent::Event(None, DrawEvent::NewFrame), "Unexpected FocusEvent")
            .run_in_scene_with_threads(&scene, test_program, 5);
    }

    #[test]
    pub fn forward_unknown_focus_event() {
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
            while let Some(focus_event) = input.next().await {
                let focus_event: FocusEvent = focus_event;
                test_program.send(focus_event).await.unwrap();
            }
        }, 20);

        // Test is to send the message to the parent program and expect it to get relayed back to the test program
        TestBuilder::new()
            .send_message_to_target(parent_program, FocusEvent::Event(None, DrawEvent::NewFrame))
            .expect_message_matching(FocusEvent::Event(None, DrawEvent::NewFrame), "Unexpected FocusEvent")
            .run_in_scene_with_threads(&scene, test_program, 5);
    }
}