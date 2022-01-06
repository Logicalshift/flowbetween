use crate::model::*;
use crate::sidebar::panel::*;
use crate::sidebar::panel_style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;
use flo_canvas_animation::description::*;

use futures::prelude::*;

use std::sync::*;

///
/// Creates the binding that indicates if the repeat sidebar panel is active or not
///
fn repeat_panel_active<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> BindRef<bool> {
    let selected_sub_effect = model.selection().selected_sub_effect.clone();

    computed(move || {
        if let Some((_elem_id, subeffect)) = selected_sub_effect.get() {
            // Sub-effect must be a repeat element
            match subeffect.effect_description() {
                EffectDescription::Repeat(_, _) => true,
                _                               => false
            }
        } else {
            // No sub-effect selected
            false
        }
    }).into()
}

fn repeat_panel_ui<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>, length_units: &Binding<TimeUnits>, height: &Binding<f64>) -> BindRef<Control> {
    // Copy the parts of the model we need
    let selected_sub_effect = model.selection().selected_sub_effect.clone();
    let length_units        = length_units.clone();
    let frame_length        = model.timeline().frame_duration.clone();

    // Compute some derived values
    let length_units_2      = length_units.clone();
    let repeat_length       = computed(move || {
        let sub_effect      = selected_sub_effect.get();

        sub_effect.and_then(|(_, sub_effect)|
            match sub_effect.effect_description() {
                EffectDescription::Repeat(length, _) => {
                    let length_units    = length_units_2.get();
                    let frame_length    = frame_length.get();

                    Some(format!("{:.2}", length_units.from_duration(*length, frame_length)))
                },

                _ => None
            })
    });

    // Set the height of the panel according to how many parts we need
    height.set((2.0*PANEL_VERT_PADDING + 2.0*PANEL_LABEL_HEIGHT) as f64);

    computed(move || {
        let length_units    = length_units.get();
        let repeat_length   = repeat_length.get();

        if let Some(repeat_length) = repeat_length {
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::empty()
                        .with(Bounds::next_vert(PANEL_VERT_PADDING)),
                    Control::container()
                        .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                        .with(vec![
                            Control::label()
                                .with(TextAlign::Right)
                                .with("Repeat every:")
                                .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                            Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                            Control::text_box()
                                .with(Bounds::next_horiz(PANEL_TEXT_WIDTH))
                                .with((ActionTrigger::SetValue, "SetRepeat"))
                                .with(repeat_length),
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
                    Control::container()
                        .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                        .with(ControlAttribute::Padding((40, 4), (40, 1)))
                        .with(vec![
                            Control::button()
                                .with(Bounds::fill_all())
                                .with(vec![
                                    Control::label()
                                        .with(Bounds::fill_all())
                                        .with(TextAlign::Center)
                                        .with((ActionTrigger::SetValue, "RepeatAfterCurrentFrame"))
                                        .with("Repeat after current frame")
                                ])
                        ]),
                    Control::empty()
                        .with(Bounds::next_vert(PANEL_VERT_PADDING)),
                ])
        } else {
            // Not a repeat effect
            Control::empty()
        }
    }).into()
}

///
/// Creates the 'repeat effect' animation sidebar panel
///
pub fn animation_repeat_sidebar_panel<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> SidebarPanel {
    // Set up the model
    let model           = Arc::clone(model);
    let is_active       = repeat_panel_active(&model);
    let height          = bind(128.0);
    let length_units    = bind(TimeUnits::Seconds);
    let cntrl_height    = height.clone();

    // Create a new immediate controller
    let controller = ImmediateController::empty(move |events, actions, _resources| {
        let model           = Arc::clone(&model);
        let height          = cntrl_height.clone();
        let length_units    = length_units.clone();

        async move {
            let mut events  = events;
            let mut actions = actions;

            // Set up the UI
            let ui          = repeat_panel_ui(&model, &length_units, &height);
            actions.send(ControllerAction::SetUi(ui)).await.ok();

            // Run the controller
            while let Some(event) = events.next().await {

            }
        }
    });

    SidebarPanel::with_title("Animation: Repeat")
        .with_active(is_active)
        .with_controller(controller)
        .with_height(height)
}
