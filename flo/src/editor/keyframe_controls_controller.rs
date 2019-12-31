use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use ::desync::*;

use std::sync::*;
use std::time::Duration;

///
/// Provides the buttons for controlling the keyframes
///
pub struct KeyFrameControlsController<Anim: 'static+Animation+EditableAnimation> {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>,

    /// The view model for this controller
    view_model: Arc<DynamicViewModel>,

    /// The frame model
    frame: FrameModel,

    /// The onion skin model
    onion_skin: OnionSkinModel<Anim>,

    /// The timeline model
    timeline: TimelineModel<Anim>,

    /// The current frame binding
    current_time: Binding<Duration>,

    // The currently selected layer ID
    selected_layer: Binding<Option<u64>>,

    /// The edit sink for the animation
    edit_sink: Desync<Publisher<Arc<Vec<AnimationEdit>>>>,

    debug_model: FloModel<Anim>
}

impl<Anim: 'static+Animation+EditableAnimation> KeyFrameControlsController<Anim> {
    ///
    /// Creates a new keyframes controls controller
    ///
    pub fn new(model: &FloModel<Anim>) -> KeyFrameControlsController<Anim> {
        // Create the viewmodel
        let frame       = model.frame();
        let timeline    = model.timeline();
        let onion_skin  = model.onion_skin();
        let view_model  = Arc::new(DynamicViewModel::new());

        let selected_layer          = timeline.selected_layer.clone();
        let create_keyframe_on_draw = frame.create_keyframe_on_draw.clone();
        let show_onion_skins        = onion_skin.show_onion_skins.clone();
        let keyframe_selected       = frame.keyframe_selected.clone();
        let prev_next_1             = frame.previous_and_next_keyframe.clone();
        let prev_next_2             = frame.previous_and_next_keyframe.clone();

        view_model.set_computed("CreateKeyFrameOnDrawSelected", move || PropertyValue::Bool(create_keyframe_on_draw.get()));
        view_model.set_computed("ShowOnionSkinsSelected",       move || PropertyValue::Bool(show_onion_skins.get()));
        view_model.set_computed("CanCreateKeyFrame",            move || PropertyValue::Bool(selected_layer.get().is_some() && !keyframe_selected.get()));
        view_model.set_computed("CanMoveToPreviousKeyFrame",    move || PropertyValue::Bool(prev_next_1.get().0.is_some()));
        view_model.set_computed("CanMoveToNextKeyFrame",        move || PropertyValue::Bool(prev_next_2.get().1.is_some()));

        // The edit sink lets us send edits to the animation (in particular, the 'new keyframe' edits)
        let edit_sink       = model.edit();

        // Create the images and the UI
        let images          = Arc::new(Self::images());
        let ui              = Self::ui(Arc::clone(&images));

        KeyFrameControlsController {
            ui:             ui,
            images:         images,
            view_model:     view_model,
            frame:          frame.clone(),
            onion_skin:     onion_skin.clone(),
            timeline:       timeline.clone(),
            current_time:   timeline.current_time.clone(),
            selected_layer: timeline.selected_layer.clone(),
            edit_sink:      Desync::new(edit_sink),
            debug_model:    model.clone()
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
                                .with(vec![Control::empty().with(previous_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((9, 4), (4, 4)))
                                .with(State::Enabled(Property::bound("CanMoveToPreviousKeyFrame")))
                                .with((ActionTrigger::Click, "MoveToPreviousKeyFrame"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::empty().with(new_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Enabled(Property::bound("CanCreateKeyFrame")))
                                .with((ActionTrigger::Click, "CreateKeyFrame"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::empty().with(onion_skins).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Selected(Property::bound("ShowOnionSkinsSelected")))
                                .with(State::Enabled(Property::Bool(true)))
                                .with((ActionTrigger::Click, "ToggleShowOnionSkins"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::empty().with(new_on_paint).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((4, 4), (4, 4)))
                                .with(State::Selected(Property::bound("CreateKeyFrameOnDrawSelected")))
                                .with(State::Enabled(Property::Bool(true)))
                                .with((ActionTrigger::Click, "ToggleCreateKeyFrameOnDraw"))
                                .with(Bounds::next_horiz(22.0)),

                            Control::button()
                                .with(vec![Control::empty().with(next_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
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

impl<Anim: 'static+Animation+EditableAnimation> Controller for KeyFrameControlsController<Anim> {
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
                self.frame.create_keyframe_on_draw.set(!current_value);
            },

            "ToggleShowOnionSkins" => {
                let current_value = self.onion_skin.show_onion_skins.get();
                self.onion_skin.show_onion_skins.set(!current_value);

                let current_time        = self.current_time.get();
                let selected_layer      = self.selected_layer.get();

                // Debugging: remove the cache for the current frame if there is one
                if let Some(selected_layer) = selected_layer {
                    self.debug_model.get_layer_with_id(selected_layer)
                        .map(|layer| layer
                            .get_canvas_cache_at_time(current_time)
                            .invalidate(CacheType::OnionSkinLayer));
                }
            },

            "MoveToPreviousKeyFrame" => {
                let previous_frame = self.frame.previous_and_next_keyframe.get().0;
                if let Some(previous_frame) = previous_frame {
                    self.current_time.set(previous_frame);
                }
            },

            "MoveToNextKeyFrame" => {
                let next_frame = self.frame.previous_and_next_keyframe.get().1;
                if let Some(next_frame) = next_frame {
                    self.current_time.set(next_frame);
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
                        let _ = self.edit_sink.future(move |edit_sink| edit_sink.publish(Arc::new(vec![
                            AnimationEdit::Layer(selected_layer, LayerEdit::AddKeyFrame(current_time))
                        ])));
                        self.edit_sink.sync(|_| { });

                        // Invalidate the canvas
                        self.timeline.invalidate_canvas();
                        self.timeline.update_keyframe_bindings();
                    }
                }
            },

            _ => { }
        }
    }
}
