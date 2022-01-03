use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// Creates the document settings controller and UI
///
/// This is used to set things like the size of the document and the frame rate
///
pub fn document_settings_controller<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> impl Controller {
    // Sizes of various things
    let label_width     = 72.0;
    let label_height    = 26.0;
    let label_gap       = 8.0;
    let text_width      = 40.0;
    let vert_padding    = 12.0;

    // Create the controller to run the document settings panel
    let model           = Arc::clone(model);

    ImmediateController::empty(move |events, actions, _resources| {
        let model       = Arc::clone(&model);
        let mut actions = actions;

        async move {
            // Set up the viewmodel bindings

            // Set up the UI
            let size            = model.size.clone();
            let frame_duration  = model.timeline().frame_duration.clone();
            let duration        = model.timeline().duration.clone();
            let ui              = computed(move || {
                Control::container()
                    .with(Bounds::fill_all())
                    .with(vec![
                        Control::empty().with(Bounds::next_vert(vert_padding)),
                        Control::container()
                            .with(Bounds::next_vert(label_height))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Width:")
                                    .with(Bounds::next_horiz(label_width)),
                                Control::empty().with(Bounds::next_horiz(label_gap)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(text_width))
                                    .with("1920"),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("pixels")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(label_height))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Height:")
                                    .with(Bounds::next_horiz(label_width)),
                                Control::empty().with(Bounds::next_horiz(label_gap)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(text_width))
                                    .with("1080"),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("pixels")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(label_height))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("FPS:")
                                    .with(Bounds::next_horiz(label_width)),
                                Control::empty().with(Bounds::next_horiz(label_gap)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(text_width))
                                    .with("30"),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("frames/second")
                            ]),
                        Control::empty().with(Bounds::next_vert(vert_padding)),
                    ])
            });

            actions.send(ControllerAction::SetUi(ui.into())).await.ok();

            // Run the events
            let mut events = events;
            while let Some(event) = events.next().await {

            }
        }
    })
}
