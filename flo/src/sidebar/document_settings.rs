use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::time::{Duration};

///
/// Creates the document settings controller and UI
///
/// This is used to set things like the size of the document and the frame rate
///
pub fn document_settings_controller<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>, height: Binding<f64>) -> impl Controller {
    // Maximum values for various things
    let max_width       = 10000;
    let max_height      = 10000;
    let max_fps         = 1000;
    let max_length      = Duration::from_secs(24 * 60 * 60);

    // Minimum values for various hings
    let min_width       = 1;
    let min_height      = 1;
    let min_fps         = 1;
    let min_length      = Duration::from_millis(1);

    // Sizes of various things
    let label_width     = 72.0;
    let label_height    = 26.0;
    let label_gap       = 8.0;
    let text_width      = 64.0;
    let vert_padding    = 12.0;

    // Create the controller to run the document settings panel
    let model           = Arc::clone(model);

    ImmediateController::empty(move |events, actions, _resources| {
        let model       = Arc::clone(&model);
        let height      = height.clone();
        let mut actions = actions;

        async move {
            // Set up the viewmodel bindings

            // Set up the UI
            let size            = model.size.clone();
            let frame_duration  = model.timeline().frame_duration.clone();
            let duration        = model.timeline().duration.clone();
            let ui              = computed(move || {
                // Get the values for the labels
                let (size_x, size_y)    = size.get();
                let frame_duration      = frame_duration.get();
                let duration            = duration.get();

                // Convert to strings
                let size_x              = format!("{}", size_x.floor() as u64);
                let size_y              = format!("{}", size_y.floor() as u64);
                let fps                 = 1_000_000_000.0 / (frame_duration.as_nanos() as f64);
                let fps                 = fps.floor() as u64;
                let fps                 = format!("{}", fps);
                let duration            = (duration.as_nanos() as f64) / 1_000_000_000.0;
                let duration            = format!("{:.2}", duration);

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
                                    .with((ActionTrigger::SetValue, "SetWidth"))
                                    .with(size_x),
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
                                    .with((ActionTrigger::SetValue, "SetHeight"))
                                    .with(size_y),
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
                                    .with("Frame rate:")
                                    .with(Bounds::next_horiz(label_width)),
                                Control::empty().with(Bounds::next_horiz(label_gap)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(text_width))
                                    .with((ActionTrigger::SetValue, "SetFps"))
                                    .with(fps),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("frames/second")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(label_height))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Length:")
                                    .with(Bounds::next_horiz(label_width)),
                                Control::empty().with(Bounds::next_horiz(label_gap)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(text_width))
                                    .with((ActionTrigger::SetValue, "SetDuration"))
                                    .with(duration),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::container()
                                    .with(Bounds::next_horiz(96.0))
                                    .with(ControlAttribute::Padding((2, 2), (2, 2)))
                                    .with(vec![
                                        Control::combo_box()
                                            .with(Bounds::fill_all())
                                            .with("seconds")
                                            .with(vec![
                                                Control::label().with("frames"),
                                                Control::label().with("seconds"),
                                                Control::label().with("minutes"),
                                            ])
                                    ])
                            ]),
                        Control::empty().with(Bounds::next_vert(vert_padding)),
                    ])
            });

            actions.send(ControllerAction::SetUi(ui.into())).await.ok();
            height.set(4.0 * (label_height as f64) + 2.0 * (vert_padding as f64));

            // Run the events
            let mut events = events;
            while let Some(event) = events.next().await {
                match event {
                    ControllerEvent::Action(name, ActionParameter::Value(PropertyValue::String(new_value))) => {
                        match name.as_str() {
                            "SetWidth"      => { 
                                if let Ok(new_width) = u64::from_str_radix(new_value.as_str(), 10) {
                                    if new_width <= max_width && new_width >= min_width {
                                        // Keep the height the same
                                        let (_, height) = model.size.get();

                                        // Update the width in the model
                                        model.edit().publish(Arc::new(vec![AnimationEdit::SetSize(new_width as _, height)])).await;
                                    }
                                }
                            },
                            "SetHeight"     => {
                                if let Ok(new_height) = u64::from_str_radix(new_value.as_str(), 10) {
                                    if new_height <= max_height && new_height >= min_height {
                                        // Keep the width the same
                                        let (width, _) = model.size.get();

                                        // Update the height in the model
                                        model.edit().publish(Arc::new(vec![AnimationEdit::SetSize(width, new_height as _)])).await;
                                    }
                                }
                            },
                            "SetFps"        => {
                                if let Ok(new_fps) = u64::from_str_radix(new_value.as_str(), 10) {
                                    if new_fps <= max_fps && new_fps >= min_fps {
                                        // Update the frame rate in the model
                                        let frame_duration = 1_000_000_000 / new_fps;
                                        let frame_duration = Duration::from_nanos(frame_duration as _);

                                        model.edit().publish(Arc::new(vec![AnimationEdit::SetFrameLength(frame_duration)])).await;
                                    }
                                }
                            },
                            "SetDuration"   => { }

                            _ => {}
                        }
                    }

                    _ => { }
                }
            }
        }
    })
}
