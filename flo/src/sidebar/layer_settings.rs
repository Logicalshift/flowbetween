use super::panel_style::*;
use crate::model::*;

use futures::prelude::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// Creates the user interface for the layer settings UI
///
pub fn layer_settings_ui<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>) -> BindRef<Control> {
    let model = Arc::clone(model);

    computed(move || {
        // Find the selected layer
        let selected_layer_id   = model.timeline().selected_layer.get();
        let selected_layer      = selected_layer_id.and_then(|layer_id|
            model.timeline().layers.get()
                .iter()
                .filter(|layer| Some(layer.id) == selected_layer_id)
                .nth(0)
                .cloned());

        if let Some(selected_layer) = selected_layer {
            // Read the layer information
            let name    = selected_layer.name.get();
            let alpha   = selected_layer.alpha.get();

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
                                .with("Name:")
                                .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                            Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                            Control::text_box()
                                .with(Bounds::next_horiz(120.0))
                                .with((ActionTrigger::SetValue, "SetName"))
                                .with(name),
                        ]),
                    Control::container()
                        .with(Bounds::next_vert(PANEL_LABEL_HEIGHT))
                        .with(vec![
                            Control::label()
                                .with(TextAlign::Right)
                                .with("Opacity:")
                                .with(Bounds::next_horiz(PANEL_LABEL_WIDTH)),
                            Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                            Control::slider()
                                .with(Bounds::next_horiz(120.0))
                                .with(State::Range((0.0.to_property(), 1.0.to_property())))
                                .with(State::Value(Property::Bind("Alpha".to_string())))
                                .with((ActionTrigger::EditValue, "EditAlphaSlider"))
                                .with((ActionTrigger::SetValue, "SetAlphaSlider")),
                            Control::empty().with(Bounds::next_horiz(PANEL_LABEL_GAP)),
                            Control::text_box()
                                .with(Bounds::next_horiz(36.0))
                                .with((ActionTrigger::SetValue, "SetAlphaText"))
                                .with(format!("{:.0}%", alpha * 100.0)),
                        ]),

                    Control::empty()
                        .with(Bounds::next_vert(PANEL_VERT_PADDING)),
                ])
        } else {
            // No selected layer
            Control::label()
                .with(Bounds::fill_all())
                .with("No selected layer")
        }
    }).into()
}


///
/// Creates the UI for editing the settings for the currently selected layer
///
pub fn layer_settings_controller<Anim: 'static+Animation+EditableAnimation>(model: &Arc<FloModel<Anim>>, height: Binding<f64>) -> impl Controller {
    let model = Arc::clone(model);

    ImmediateController::empty(move |events, actions, _resources| {
        let mut events  = events;
        let mut actions = actions;
        let height      = height.clone();
        let model       = model.clone();

        async move {
            // Create a binding for the current alpha value
            let selected_layer_id   = model.timeline().selected_layer.clone();
            let layers              = model.timeline().layers.clone();
            let layer_alpha         = computed(move || {
                let selected_layer_id   = selected_layer_id.get();
                let selected_layer      = selected_layer_id.and_then(|layer_id|
                    layers.get()
                        .iter()
                        .filter(|layer| Some(layer.id) == selected_layer_id)
                        .nth(0)
                        .cloned());

                let alpha = selected_layer
                    .map(|layer| layer.alpha.get())
                    .unwrap_or(1.0);

                PropertyValue::Float(alpha)
            });

            // Set up the UI
            let ui  = layer_settings_ui(&model);
            height.set((2.0*PANEL_VERT_PADDING + 2.0*PANEL_LABEL_HEIGHT) as f64);

            actions.send(ControllerAction::SetPropertyBinding("Alpha".to_string(), BindRef::from(&layer_alpha))).await.ok();
            actions.send(ControllerAction::SetUi(ui)).await.ok();

            while let Some(event) = events.next().await {
                match event {
                    ControllerEvent::Action(name, parameter) => {
                        match (name.as_str(), parameter) {
                            ("EditAlphaSlider", ActionParameter::Value(PropertyValue::Float(new_alpha))) => {
                                // Set the alpha in the selected layer (temporarily)
                                let selected_layer_id   = model.timeline().selected_layer.get();
                                let selected_layer      = model.timeline().layers.get()
                                    .into_iter()
                                    .filter(|layer| Some(layer.id) == selected_layer_id)
                                    .nth(0);

                                selected_layer.map(|layer| {
                                    layer.alpha.set(new_alpha);
                                });
                            }

                            ("SetAlphaSlider", ActionParameter::Value(PropertyValue::Float(new_alpha))) => {
                                // Set the alpha in the selected layer (by editing)
                                let selected_layer_id   = model.timeline().selected_layer.get();
                                if let Some(layer_id) = selected_layer_id {
                                    model.edit().publish(Arc::new(vec![AnimationEdit::Layer(layer_id, LayerEdit::SetAlpha(new_alpha))])).await;
                                }
                            }

                            ("SetAlphaText", ActionParameter::Value(PropertyValue::String(new_alpha))) => {

                            }

                            ("SetName", ActionParameter::Value(PropertyValue::String(new_name))) => {

                            }

                            _ => { }
                        }
                    }
                }
            }
        }
    })
}
