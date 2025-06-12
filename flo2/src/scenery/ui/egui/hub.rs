use super::dialog_egui::*;
use crate::scenery::ui::dialog::*;
use crate::scenery::ui::dialog_id::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas as canvas;
use futures::prelude::*;

use std::collections::{HashMap};

///
/// Creates dialogs using egui for rendering
///
pub async fn dialog_egui_hub(input_stream: InputStream<Dialog>, context: SceneContext) {
    // Details about where to render on the canvas
    // TODO: re-use layers when dialog program end
    let mut dialog_namespace    = canvas::NamespaceId::new();
    let mut next_layer_id       = 0;

    // Hashmap mapping dialog IDs to the subprograms where they're running
    // TODO: remove from this list when dialog programs end
    let mut dialog_subprograms  = HashMap::new();

    async fn send_to_dialog(dialog: DialogId, message: Dialog, subprograms: &mut HashMap<DialogId, OutputSink<Dialog>>) {
        // Fetch the sink where messages for this dialog go
        if let Some(sink) = subprograms.get_mut(&dialog) {
            // Try sending the message
            let result = sink.send(message).await;

            // Errors probably indicate the dialog program has stopped, so remove from the list of subprograms if this happens
            if result.is_err() {
                subprograms.remove(&dialog);
            }
        }
    }

    // Read events
    let mut input_stream = input_stream;

    while let Some(dialog) = input_stream.next().await {
        use Dialog::*;

        match dialog {
            CreateDialog(dialog_id, target_program_id, bounds) => {
                // TODO: deal with dialog_id already existing

                // Assign a layer for this dialog
                let layer_id    = canvas::LayerId(next_layer_id);
                next_layer_id   += 1;

                // Assign a program ID for the new dialog
                let program_id = SubProgramId::new();

                // Start a program to run this dialog
                context.send_message(SceneControl::start_program(program_id, move |input, context| dialog_egui(input, context, dialog_namespace, layer_id, bounds), 20)).await.ok();

                // Send to this dialog
                let sink = context.send::<Dialog>(program_id).ok();
                if let Some(sink) = sink {
                    dialog_subprograms.insert(dialog_id, sink);
                }
            }

            RemoveDialog(dialog_id)                                         => { send_to_dialog(dialog_id, RemoveDialog(dialog_id), &mut dialog_subprograms).await; dialog_subprograms.remove(&dialog_id); }
            MoveDialog(dialog_id, bounds)                                   => { send_to_dialog(dialog_id, MoveDialog(dialog_id, bounds), &mut dialog_subprograms).await; }
            AddControl(dialog_id, control_id, bounds, control_type, value)  => { send_to_dialog(dialog_id, AddControl(dialog_id, control_id, bounds, control_type, value), &mut dialog_subprograms).await; }
            SetControlValue(dialog_id, control_id, value)                   => { send_to_dialog(dialog_id, SetControlValue(dialog_id, control_id, value), &mut dialog_subprograms).await; }
            MoveControl(dialog_id, control_id, bounds)                      => { send_to_dialog(dialog_id, MoveControl(dialog_id, control_id, bounds), &mut dialog_subprograms).await; }
            SetVisible(dialog_id, control_id, visible)                      => { send_to_dialog(dialog_id, SetVisible(dialog_id, control_id, visible), &mut dialog_subprograms).await; }
        }
    }
}