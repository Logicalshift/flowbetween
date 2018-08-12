use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use desync::*;
use futures::*;
use futures::executor;
use futures::executor::Spawn;

use std::sync::*;
use std::time::Duration;

///
/// Provides the buttons for controlling the keyframes
/// 
pub struct KeyFrameControlsController {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>,

    /// The view model for this controller
    view_model: Arc<DynamicViewModel>,

    /// The frame model
    frame: FrameModel,

    /// The current frame binding
    current_time: Binding<Duration>,

    // The currently selected layer ID
    selected_layer: Binding<Option<u64>>,

    /// The edit sink for the animation
    edit_sink: Desync<Spawn<Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send>>>,
}

impl KeyFrameControlsController {
    ///
    /// Creates a new keyframes controls controller
    /// 
    pub fn new<Anim: 'static+Animation+EditableAnimation>(model: &FloModel<Anim>) -> KeyFrameControlsController {
        // Create the viewmodel
        let frame       = model.frame();
        let timeline    = model.timeline();
        let view_model  = Arc::new(DynamicViewModel::new());

        let selected_layer          = timeline.selected_layer.clone();
        let create_keyframe_on_draw = frame.create_keyframe_on_draw.clone();
        let show_onion_skins        = frame.show_onion_skins.clone();
        let keyframe_selected       = frame.keyframe_selected.clone();
        let prev_next_1             = frame.previous_and_next_keyframe.clone();
        let prev_next_2             = frame.previous_and_next_keyframe.clone();

        view_model.set_computed("CreateKeyFrameOnDrawSelected", move || PropertyValue::Bool(create_keyframe_on_draw.get()));
        view_model.set_computed("ShowOnionSkinsSelected",       move || PropertyValue::Bool(show_onion_skins.get()));
        view_model.set_computed("CanCreateKeyFrame",            move || PropertyValue::Bool(selected_layer.get().is_some() && !keyframe_selected.get()));
        view_model.set_computed("CanMoveToPreviousKeyFrame",    move || PropertyValue::Bool(prev_next_1.get().0.is_some()));
        view_model.set_computed("CanMoveToNextKeyFrame",        move || PropertyValue::Bool(prev_next_2.get().1.is_some()));

        // The edit sink lets us send edits to the animation (in particular, the 'new keyframe' edits)
        let edit_sink       = executor::spawn(model.edit());

        // Create the images and the UI
        let images          = Arc::new(Self::images());
        let ui              = Self::ui(Arc::clone(&images));

        KeyFrameControlsController {
            ui:             ui,
            images:         images,
            view_model:     view_model,
            frame:          frame.clone(),
            current_time:   timeline.current_time.clone(),
            selected_layer: timeline.selected_layer.clone(),
            edit_sink:      Desync::new(edit_sink)
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui(images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        // Fetch the icon images
        let new_key_frame       = images.get_named_resource("new_key_frame").unwrap();
        let new_on_paint        = images.get_named_resource("new_on_paint").unwrap();
        let next_key_frame      = images.get_named_resource("next_key_frame").unwrap();
        let onion_skins         = images.get_named_resource("onion_skins").unwrap();
        let previous_key_frame  = images.get_named_resource("previous_key_frame").unwrap();

        // Get the parts of the model we want to use

        // Create the UI
        let ui = computed(move || {
            let new_key_frame       = new_key_frame.clone();
            let new_on_paint        = new_on_paint.clone();
            let next_key_frame      = next_key_frame.clone();
            let onion_skins         = onion_skins.clone();
            let previous_key_frame  = previous_key_frame.clone();

            Control::container()
                .with(vec![
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_LAYERS))
                        .with(Bounds::next_horiz(1.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(3.0)),
                    Control::label()
                        .with("Keyframes:")
                        .with(TextAlign::Right)
                        .with(Bounds::stretch_horiz(1.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(10.0)),
                    Control::container()
                        .with(Hint::Class("button-group".to_string()))
                        .with(vec![
                            Control::button()
                                .with(vec![Control::label().with(previous_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((9, 4), (4, 4)))
                                .with(State::Enabled(Property::bound("CanMoveToPreviousKeyFrame")))
                                .with((ActionTrigger::Click, "MoveToPreviousKeyFrame"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::label().with(new_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Enabled(Property::bound("CanCreateKeyFrame")))
                                .with((ActionTrigger::Click, "CreateKeyFrame"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::label().with(onion_skins).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Selected(Property::bound("ShowOnionSkinsSelected")))
                                .with(State::Enabled(Property::Bool(true)))
                                .with((ActionTrigger::Click, "ToggleShowOnionSkins"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::label().with(new_on_paint).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Selected(Property::bound("CreateKeyFrameOnDrawSelected")))
                                .with(State::Enabled(Property::Bool(true)))
                                .with((ActionTrigger::Click, "ToggleCreateKeyFrameOnDraw"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::label().with(next_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (9, 4)))
                                .with(State::Enabled(Property::bound("CanMoveToNextKeyFrame")))
                                .with((ActionTrigger::Click, "MoveToNextKeyFrame"))
                                .with(Bounds::next_horiz(22.0)),
                        ])
                        .with(Bounds::next_horiz(22.0*5.0))

                ])
                .with(Bounds::fill_all())
        });

        // Turn into a bindref
        BindRef::from(ui)
    }

    ///
    /// Creates the image resource manager for this controller
    /// 
    fn images() -> ResourceManager<Image> {
        let images              = ResourceManager::new();

        let new_key_frame       = images.register(svg_static(include_bytes!("../../svg/keyframes/new_key_frame.svg")));
        let new_on_paint        = images.register(svg_static(include_bytes!("../../svg/keyframes/new_on_paint.svg")));
        let next_key_frame      = images.register(svg_static(include_bytes!("../../svg/keyframes/next_key_frame.svg")));
        let onion_skins         = images.register(svg_static(include_bytes!("../../svg/keyframes/onion_skins.svg")));
        let previous_key_frame  = images.register(svg_static(include_bytes!("../../svg/keyframes/previous_key_frame.svg")));

        images.assign_name(&new_key_frame,      "new_key_frame");
        images.assign_name(&new_on_paint,       "new_on_paint");
        images.assign_name(&next_key_frame,     "next_key_frame");
        images.assign_name(&onion_skins,        "onion_skins");
        images.assign_name(&previous_key_frame, "previous_key_frame");

        images
    }
}

impl Controller for KeyFrameControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "ToggleCreateKeyFrameOnDraw" => {
                let current_value = self.frame.create_keyframe_on_draw.get();
                self.frame.create_keyframe_on_draw.clone().set(!current_value);
            },

            "ToggleShowOnionSkins" => {
                let current_value = self.frame.show_onion_skins.get();
                self.frame.show_onion_skins.clone().set(!current_value);
            },

            "MoveToPreviousKeyFrame" => { 
                let previous_frame = self.frame.previous_and_next_keyframe.get().0;
                if let Some(previous_frame) = previous_frame {
                    self.current_time.clone().set(previous_frame);
                }
            },

            "MoveToNextKeyFrame" => { 
                let next_frame = self.frame.previous_and_next_keyframe.get().1;
                if let Some(next_frame) = next_frame {
                    self.current_time.clone().set(next_frame);
                }
            },

            "CreateKeyFrame" => { 
                let current_time        = self.current_time.get();
                let selected_layer      = self.selected_layer.get();
                let keyframe_selected   = self.frame.keyframe_selected.get();

                // If we can create a keyframe (got a current layer and no keyframe selected)
                if let Some(selected_layer) = selected_layer {
                    if !keyframe_selected {
                        // Send a new keyframe edit request at the current time
                        self.edit_sink.sync(|edit_sink| edit_sink.wait_send(vec![
                            AnimationEdit::Layer(selected_layer, LayerEdit::AddKeyFrame(current_time))
                        ])).unwrap();
                    }
                }
            },

            _ => { }
        }
    }
}