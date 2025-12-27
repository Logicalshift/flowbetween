use super::focus::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;

///
/// Runs a gesture program, forwarding any focus events originally destined for the current program to the new program
///
pub async fn run_gesture_program(context: SceneContext) {
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
    // 
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
