use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

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

    /// Model binding: the 'create frame on draw' binding
    create_keyframe_on_draw: Binding<bool>,

    /// Model binding: the 'show onion skins' binding
    show_onion_skins: Binding<bool>
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
        let next_keyframe           = frame.next_keyframe.clone();
        let previous_keyframe       = frame.previous_keyframe.clone();

        view_model.set_computed("CreateKeyFrameOnDrawSelected", move || PropertyValue::Bool(create_keyframe_on_draw.get()));
        view_model.set_computed("ShowOnionSkinsSelected",       move || PropertyValue::Bool(show_onion_skins.get()));
        view_model.set_computed("CanCreateKeyFrame",            move || PropertyValue::Bool(selected_layer.get().is_some() && !keyframe_selected.get()));
        view_model.set_computed("CanMoveToNextKeyFrame",        move || PropertyValue::Bool(next_keyframe.get().is_some()));
        view_model.set_computed("CanMoveToPreviousKeyFrame",    move || PropertyValue::Bool(previous_keyframe.get().is_some()));

        // Create the images and the UI
        let images  = Arc::new(Self::images());
        let ui      = Self::ui(model, Arc::clone(&images));

        KeyFrameControlsController {
            ui:                         ui,
            images:                     images,
            view_model:                 view_model,
            create_keyframe_on_draw:    frame.create_keyframe_on_draw.clone(),
            show_onion_skins:           frame.show_onion_skins.clone()
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>, images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
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
                let current_value = self.create_keyframe_on_draw.get();
                self.create_keyframe_on_draw.clone().set(!current_value);
            },

            "ToggleShowOnionSkins" => {
                let current_value = self.show_onion_skins.get();
                self.show_onion_skins.clone().set(!current_value);
            },

            "MoveToPreviousKeyFrame" => { },
            "MoveToNextKeyFrame" => { },
            "CreateKeyFrame" => { },

            _ => { }
        }
    }
}