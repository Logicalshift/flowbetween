use super::timeline_controller::*;
use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Controller class that displays and edits the layer names
/// 
pub struct TimelineLayerListController {
    /// The user interface binding for this controller
    ui: BindRef<Control>,

    /// The currently selected layer
    selected_layer_id: Binding<Option<u64>>
}

impl TimelineLayerListController {
    ///
    /// Creates a new timeline layer controller
    /// 
    pub fn new<Anim: 'static+Animation>(model: &FloModel<Anim>) -> TimelineLayerListController {
        // Create the UI from the model
        let ui                  = Self::ui(model);
        let selected_layer_id   = model.timeline().selected_layer.clone();

        TimelineLayerListController {
            ui:                 ui,
            selected_layer_id:  selected_layer_id
        }
    }

    ///
    /// Creates a control from a layer model
    /// 
    fn layer_label(model: &LayerModel, selected_layer_id: Option<u64>) -> Control {
        let name        = model.name.get();
        let layer_id    = model.id;

        let is_selected = Some(layer_id) == selected_layer_id;
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
                Control::label()
                    .with(name)
                    .with(Bounds::stretch_horiz(1.0))
                    .with((ActionTrigger::Click, format!("SelectLayer-{}", layer_id)))
            ])
    }

    ///
    /// Creates the UI binding from the model
    /// 
    fn ui<Anim: 'static+Animation>(model: &FloModel<Anim>) -> BindRef<Control> {
        // Extract the bindings we're going to use from the model
        let layers          = model.timeline().layers.clone();
        let selected_layer  = model.timeline().selected_layer.clone();

        // Generate the UI
        let ui = computed(move || {
            // Fetch the layer model
            let layers          = layers.get();
            let selected_layer  = selected_layer.get();

            // Each layer creates a control
            let layer_controls = layers.into_iter()
                .flat_map(|layer_model| {
                    // The layer is a simple label
                    let label = Self::layer_label(&layer_model, selected_layer);
                        
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

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            _ => {
                // 'SelectLayer-x' should select layer 'x'
                if action_id.starts_with("SelectLayer-") {
                    // Get the layer ID that should be selected
                    let (_, layer_id)   = action_id.split_at("SelectLayer-".len());
                    let layer_id        = u64::from_str_radix(layer_id, 10).unwrap();

                    // Update the model
                    self.selected_layer_id.clone().set(Some(layer_id));
                }
            }
        }
    }
}