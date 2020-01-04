use flo_ui::*;

use std::sync::*;

///
/// Represents an input to a tool
///
#[derive(Debug)]
pub enum ToolInput<ToolData> {
    /// Specifies that this tool has been selected
    Select,

    /// Specifies that this tool has been deselected
    Deselect,

    /// Specifies the data set for this tool
    Data(Arc<ToolData>),

    /// Specifies painting on a specific device
    PaintDevice(PaintDevice),

    /// Specifies an input paint action
    Paint(Painting)
}

impl<ToolData> ToolInput<ToolData> {
    ///
    /// Given a set of toolinput actions, ensures that only the final 'continue' action is
    /// left in for any given paint device.
    ///
    /// This is useful for actions like dragging a selection around where only the most
    /// recent action needs to be processed.
    ///
    pub fn last_paint_actions_only<Actions: IntoIterator<Item=ToolInput<ToolData>>>(actions: Actions) -> impl Iterator<Item=ToolInput<ToolData>> {
        // For the moment we just build up a vector of results
        let mut result = vec![];

        // Initially there's no last continue action
        let mut last_continue = None;

        // Process the actions
        for action in actions {
            use self::ToolInput::*;

            match action {
                PaintDevice(_new_device) => {
                    // When changing device: push any last continue
                    if let Some(last_continue) = last_continue.take() {
                        result.push(last_continue);
                    }

                    // Action is otherwise left unchanged
                    result.push(action);
                },

                Paint(paint) => {
                    match paint.action {
                        PaintAction::Continue => {
                            // This action is suppressed and becomes the last continue action
                            last_continue = Some(action);
                        },

                        PaintAction::Prediction => {
                            // Treat predictions as if they were a continue
                            last_continue = Some(action);
                        },

                        PaintAction::Cancel => {
                            // Continues are all removed: only the cancel action is pushed
                            last_continue = None;
                            result.push(action);
                        },

                        PaintAction::Finish | PaintAction::Start => {
                            // This commits the continue
                            if let Some(last_continue) = last_continue.take() {
                                result.push(last_continue);
                            }
                            result.push(action);
                        }
                    }
                },

                other_action => {
                    // All other actions are left unchanged
                    result.push(other_action);
                }
            }
        }

        // If there's a continue waiting to be processed, add that to the result
        if let Some(last_continue) = last_continue {
            result.push(last_continue);
        }

        result.into_iter()
    }
}
