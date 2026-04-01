use super::panel_style::*;
use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::str::{FromStr};
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

    let length_units    = bind(TimeUnits::Seconds);

    // Create the controller to run the document settings panel
    let model           = Arc::clone(model);

    ImmediateController::empty(move |events, actions, _resources| {
        let model           = Arc::clone(&model);
        let height          = height.clone();
        let length_units    = length_units.clone();
        let mut actions     = actions;

        async move {
            // Set up the viewmodel bindings

            // Set up the UI
            let size            = model.size.clone();
            let frame_duration  = model.timeline().frame_duration.clone();
            let duration        = model.timeline().duration.clone();
            let length_units_2  = length_units.clone();
            let ui              = computed(move || {
                // Get the values for the labels
                let length_units        = length_units_2.get();
                let (size_x, size_y)    = size.get();
                let frame_duration      = frame_duration.get();
                let duration            = duration.get();

                // Convert to strings
                let size_x              = format!("{}", size_x.floor() as u64);
                let size_y              = format!("{}", size_y.floor() as u64);
                let fps                 = 1_000_000_000.0 / (frame_duration.as_nanos() as f64);
                let fps                 = fps.floor() as u64;
                let fps                 = format!("{}", fps);
                let duration            = length_units.from_duration(duration, frame_duration);
                let duration            = format!("{:.2}", duration);

                Control::container()
                    .with(Bounds::fill_all())
                    .with(vec![
                        Control::empty().with(Bounds::next_vert(PANEL_VERT_PADDING)),
                        Control::container()
                            .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Width:")
                                    .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                                Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(PANEL_TEXT_WIDTH))
                                    .with((ActionTrigger::SetValue, "SetWidth"))
                                    .with(size_x),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("pixels")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Height:")
                                    .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                                Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(PANEL_TEXT_WIDTH))
                                    .with((ActionTrigger::SetValue, "SetHeight"))
                                    .with(size_y),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("pixels")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Frame rate:")
                                    .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                                Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(PANEL_TEXT_WIDTH))
                                    .with((ActionTrigger::SetValue, "SetFps"))
                                    .with(fps),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::label()
                                    .with(Bounds::fill_horiz())
                                    .with("frames/second")
                            ]),
                        Control::container()
                            .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                            .with(vec![
                                Control::label()
                                    .with(TextAlign::Right)
                                    .with("Length:")
                                    .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                                Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                                Control::text_box()
                                    .with(Bounds::next_horiz(PANEL_TEXT_WIDTH))
                                    .with((ActionTrigger::SetValue, "SetDuration"))
                                    .with(duration),
                                Control::empty().with(Bounds::next_horiz(2.0)),
                                Control::container()
                                    .with(Bounds::next_horiz(96.0))
                                    .with(ControlAttribute::Padding((2, 2), (2, 2)))
                                    .with(vec![
                                        Control::combo_box()
                                            .with(Bounds::fill_all())
                                            .with(length_units.description())
                                            .with(vec![
                                                Control::label().with("frames").with((ActionTrigger::Click, "LengthFrames")),
                                                Control::label().with("seconds").with((ActionTrigger::Click, "LengthSeconds")),
                                                Control::label().with("minutes").with((ActionTrigger::Click, "LengthMinutes")),
                                            ])
                                    ])
                            ]),
                        Control::empty().with(Bounds::next_vert(PANEL_VERT_PADDING)),
                    ])
            });

            actions.send(ControllerAction::SetUi(ui.into())).await.ok();
            height.set(4.0 * (PANEL_LABEL_HEIGHT as f64) + 2.0 * (PANEL_VERT_PADDING as f64));

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
                            "SetDuration"   => {
                                if let Ok(new_length) = f64::from_str(new_value.as_str()) {
                                    let length_units    = length_units.get();
                                    let frame_duration  = model.timeline().frame_duration.get();
                                    let new_length      = length_units.to_duration(new_length, frame_duration);

                                    if new_length <= max_length && new_length >= min_length {
                                        // Update the length of the animation
                                        model.edit().publish(Arc::new(vec![AnimationEdit::SetLength(new_length)])).await;
                                    }
                                }
                            }

                            _ => {}
                        }
                    },

                    ControllerEvent::Action(name, _) => {
                        match name.as_str() {
                            "LengthFrames"  => { length_units.set(TimeUnits::Frames); }
                            "LengthSeconds" => { length_units.set(TimeUnits::Seconds); }
                            "LengthMinutes" => { length_units.set(TimeUnits::Minutes); }

                            _               => { }
                        }
                    }

                    _ => { }
                }
            }
        }
    })
}
