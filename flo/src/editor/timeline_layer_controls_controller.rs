use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use ::desync::*;

use std::sync::*;

///
/// Controller that provides controls for adding/deleting/editing layers (generally displayed above the main layer list)
///
pub struct TimelineLayerControlsController<Anim: Animation> {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The animation editing stream where this will send updates
    edit: Desync<Publisher<Arc<Vec<AnimationEdit>>>>,

    /// The animation that this will edit
    animation: Box<dyn Animation>,

    /// The timeline model we're editing
    timeline: TimelineModel<Anim>
}

impl<Anim: 'static+Animation+EditableAnimation> TimelineLayerControlsController<Anim> {
    ///
    /// Creates a new timeline layer controls controller
    ///
    pub fn new(model: &FloModel<Anim>) -> TimelineLayerControlsController<Anim> {
        let ui          = Self::ui();
        let edit        = model.edit();
        let animation   = Box::new(model.clone());
        let timeline    = model.timeline().clone();

        TimelineLayerControlsController {
            ui:         ui,
            edit:       Desync::new(edit),
            animation:  animation,
            timeline:   timeline
        }
    }

    ///
    /// Creates the UI for the layer controls controller
    ///
    fn ui() -> BindRef<Control> {
        // Create the UI
        let ui = computed(move || {
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::container()
                        .with(Font::Size(13.0))
                        .with(Font::Weight(FontWeight::ExtraBold))
                        .with(ControlAttribute::Padding((4, 2), (4, 2)))
                        .with(vec![
                            Control::empty()
                                .with(Bounds::stretch_horiz(1.0)),
                            Control::container()
                                .with(Hint::Class("button-group".to_string()))
                                .with(Bounds::next_horiz(36.0))
                                .with(vec![
                                    Control::button()
                                        .with(Bounds::next_horiz(18.0))
                                        .with((ActionTrigger::Click, "AddNewLayer"))
                                        .with(vec![
                                            Control::label()
                                                .with(Bounds::fill_all())
                                                .with(TextAlign::Center)
                                                .with("+")
                                        ]),
                                    Control::button()
                                        .with(Bounds::next_horiz(18.0))
                                        .with((ActionTrigger::Click, "RemoveLayer"))
                                        .with(vec![
                                            Control::label()
                                                .with(Bounds::fill_all())
                                                .with(TextAlign::Center)
                                                .with("-")
                                        ])
                                ])
                        ])
                        .with(Bounds::stretch_vert(1.0)),
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_BORDER))
                        .with(Bounds::next_vert(1.0))
                ])
        });

        // Turn into a bindref
        BindRef::from(ui)
    }
}

impl<Anim: 'static+Animation+EditableAnimation> Controller for TimelineLayerControlsController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "AddNewLayer" => {
                // Pick a layer ID for the new layer
                let new_layer_id = self.animation.get_layer_ids().into_iter().max().unwrap_or(0) + 1;

                // Send to the animation
                let _ = self.edit.future(move |animation| {
                    animation.publish(Arc::new(vec![
                        AnimationEdit::AddNewLayer(new_layer_id),
                    ]))
                });
                self.edit.sync(|_| {});

                // Select the new layer
                self.timeline.selected_layer.set(Some(new_layer_id));

                // Update the model
                self.timeline.update_keyframe_bindings();
                self.timeline.invalidate_canvas();
            },

            "RemoveLayer" => {
                // This will remove the selected layer
                let layer_to_remove = self.timeline.selected_layer.get();

                // Check that the layer actually exists
                let layer_ids = self.animation.get_layer_ids();
                if layer_ids.iter().any(|layer_id| Some(*layer_id) == layer_to_remove) {
                    let layer_to_remove = layer_to_remove.unwrap();

                    // Remove the layer
                    let _ = self.edit.future(move |animation| {
                        animation.publish(Arc::new(vec![
                            AnimationEdit::RemoveLayer(layer_to_remove)
                        ]))
                    });
                    self.edit.sync(|_| {});

                    // Update the model
                    self.timeline.update_keyframe_bindings();
                    self.timeline.invalidate_canvas();

                    // Select the layer after the one we just deleted (or the one before if it was the last in the list)
                    let old_layer_index = layer_ids.iter()
                        .enumerate()
                        .filter(|(_, layer_id)| **layer_id == layer_to_remove)
                        .nth(0);

                    let new_selected_layer = if let Some((old_layer_index, _)) = old_layer_index {
                        if layer_ids.len() == 1 {
                            // No layers left after the deletion
                            None
                        } else if old_layer_index+1 >= layer_ids.len() {
                            // No next layer
                            Some(layer_ids[old_layer_index-1])
                        } else {
                            // Default behaviour: pick the next layer
                            Some(layer_ids[old_layer_index+1])
                        }
                    } else {
                        None
                    };

                    self.timeline.selected_layer.set(new_selected_layer);
                }
            },

            _ => { }
        }
    }
}
