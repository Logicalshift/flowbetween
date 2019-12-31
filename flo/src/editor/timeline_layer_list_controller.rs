use super::timeline_controller::*;
use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use ::desync::*;

use std::sync::*;

///
/// Controller class that displays and edits the layer names
///
pub struct TimelineLayerListController {
    /// The user interface binding for this controller
    ui: BindRef<Control>,

    /// Where the edits are sent
    edit_sink: Desync<Publisher<Arc<Vec<AnimationEdit>>>>,

    /// The currently selected layer
    selected_layer_id: Binding<Option<u64>>,

    /// The layer whose name is being edited
    editing_layer_id: Binding<Option<u64>>
}

impl TimelineLayerListController {
    ///
    /// Creates a new timeline layer controller
    ///
    pub fn new<Anim: 'static+Animation+EditableAnimation>(model: &FloModel<Anim>) -> TimelineLayerListController {
        // Create the UI from the model
        let selected_layer_id   = model.timeline().selected_layer.clone();
        let editing_layer_id    = bind(None);
        let ui                  = Self::ui(model, BindRef::from(editing_layer_id.clone()));

        let edit_sink           = model.edit();

        TimelineLayerListController {
            ui:                 ui,
            edit_sink:          Desync::new(edit_sink),
            selected_layer_id:  selected_layer_id,
            editing_layer_id:   editing_layer_id
        }
    }

    ///
    /// Creates a control from a layer model
    ///
    fn layer_label(model: &LayerModel, selected_layer_id: Option<u64>, editing_layer_id: Option<u64>) -> Control {
        let name        = model.name.get();
        let layer_id    = model.id;

        let is_selected = Some(layer_id) == selected_layer_id;
        let is_editing  = Some(layer_id) == editing_layer_id;
        let background  = if is_selected { TIMELINE_SELECTED_LAYER } else { TIMELINE_BACKGROUND };

        Control::container()
            .with(Bounds::next_vert(TIMELINE_LAYER_HEIGHT-1.0))
            .with(ControlAttribute::Padding((4, 1), (1, 1)))
            .with(Appearance::Background(background))
            .with(vec![
                Control::empty()
                    .with(Bounds::next_horiz(20.0)),
                Control::empty()
                    .with(Bounds::next_horiz(2.0)),
                if is_editing {
                    Control::text_box()
                        .with(name)
                        .with(Bounds::stretch_horiz(1.0))
                        .with(State::FocusPriority(Property::from(128.0)))
                        .with((ActionTrigger::CancelEdit, "CancelEditingLayer"))
                        .with((ActionTrigger::Dismiss, "StopEditingLayer"))
                        .with((ActionTrigger::SetValue, "StopEditingLayer"))
                } else {
                    Control::label()
                        .with(name)
                        .with(Bounds::stretch_horiz(1.0))
                        .with((
                            ActionTrigger::Click,
                            if is_selected { format!("EditLayer-{}", layer_id) } else { format!("SelectLayer-{}", layer_id) }
                        ))
                }
            ])
    }

    ///
    /// Creates the UI binding from the model
    ///
    fn ui<Anim: 'static+Animation>(model: &FloModel<Anim>, editing_layer_id: BindRef<Option<u64>>) -> BindRef<Control> {
        // Extract the bindings we're going to use from the model
        let layers          = model.timeline().layers.clone();
        let selected_layer  = model.timeline().selected_layer.clone();

        // Generate the UI
        let ui = computed(move || {
            // Fetch the layer model
            let layers          = layers.get();
            let selected_layer  = selected_layer.get();
            let editing_layer   = editing_layer_id.get();

            // Each layer creates a control
            let layer_controls = layers.into_iter()
                .flat_map(|layer_model| {
                    // The layer is a simple label
                    let label = Self::layer_label(&layer_model, selected_layer, editing_layer);

                    // Each layer is followed by a divider
                    let divider = Control::empty()
                        .with(Appearance::Background(TIMESCALE_BORDER))
                        .with(Bounds::next_vert(1.0));

                   vec![label, divider]
                })
                .collect::<Vec<_>>();

            Control::container()
                .with(Font::Size(11.0))
                .with(Bounds::fill_all())
                .with(layer_controls)
        });

        // Turn into a bindref
        BindRef::from(ui)
    }
}

impl Controller for TimelineLayerListController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        match action_id {
            "CancelEditingLayer" => self.editing_layer_id.set(None),

            "StopEditingLayer" => {
                if let (ActionParameter::Value(PropertyValue::String(new_name)), Some(layer_id)) = (action_parameter, self.selected_layer_id.get()) {
                    // Send the 'rename layer' command to the edit sink
                    let new_name = new_name.clone();
                    let _ = self.edit_sink.future(move |edit_sink| {
                        edit_sink.publish(Arc::new(vec![
                            AnimationEdit::Layer(layer_id, LayerEdit::SetName(new_name))
                        ]))
                    });
                }

                // Stop editing the layer
                self.editing_layer_id.set(None)
            },

            _ => {
                // 'SelectLayer-x' should select layer 'x'. 'EditLayer-x' should edit layer 'x'
                if action_id.starts_with("SelectLayer-") {
                    // Get the layer ID that should be selected
                    let (_, layer_id)   = action_id.split_at("SelectLayer-".len());
                    let layer_id        = u64::from_str_radix(layer_id, 10).unwrap();

                    // Update the model
                    self.selected_layer_id.set(Some(layer_id));
                } else if action_id.starts_with("EditLayer-") {
                    // Get the layer ID that should be edited
                    let (_, layer_id)   = action_id.split_at("EditLayer-".len());
                    let layer_id        = u64::from_str_radix(layer_id, 10).unwrap();

                    // Update the model
                    self.editing_layer_id.set(Some(layer_id));
                }
            }
        }
    }
}
