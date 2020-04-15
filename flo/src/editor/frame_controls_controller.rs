use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::time::Duration;

///
/// The display style for the frame indicator text
///
#[derive(Clone, Copy, PartialEq)]
enum FrameDisplayStyle {
    TimeOffset,
    FrameNumber
}

///
/// The frame controls allows for choosing individual frames and playback
///
/// It differs from the similarly named keyframe controller in that not every frame contains
/// a keyframe
///
pub struct FrameControlsController<Anim: 'static+Animation+EditableAnimation> {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>,

    /// The view model for this controller
    view_model: Arc<DynamicViewModel>,

    /// The display style for the frames
    frame_style: Binding<FrameDisplayStyle>,

    /// The frame model
    frame: FrameModel,

    /// The timeline model
    timeline: TimelineModel<Anim>,

    /// The current frame binding
    current_time: Binding<Duration>,
}

impl<Anim: 'static+Animation+EditableAnimation> FrameControlsController<Anim> {
    ///
    /// Creates a new keyframes controls controller
    ///
    pub fn new(model: &FloModel<Anim>) -> FrameControlsController<Anim> {
        // Create the viewmodel
        let frame           = model.frame();
        let timeline        = model.timeline();
        let frame_style     = bind(FrameDisplayStyle::TimeOffset);
        let view_model      = Arc::new(DynamicViewModel::new());

        let frame_text      = Self::frame_text(model, frame_style.clone());

        // Create the images and the UI
        let images          = Arc::new(Self::images());
        let ui              = Self::ui(Arc::clone(&images), frame_text);

        FrameControlsController {
            ui:             ui,
            images:         images,
            view_model:     view_model,
            frame_style:    frame_style,
            frame:          frame.clone(),
            timeline:       timeline.clone(),
            current_time:   timeline.current_time.clone(),
        }
    }

    ///
    /// Creates the frame display text
    ///
    fn frame_text(model: &FloModel<Anim>, frame_style: Binding<FrameDisplayStyle>) -> BindRef<String> {
        // Capture data we need from the model
        let timeline        = model.timeline();
        let current_time    = timeline.current_time.clone();
        let frame_duration  = timeline.frame_duration.clone();

        // The binding itself
        BindRef::new(&computed(move || {
            match frame_style.get() {
                FrameDisplayStyle::TimeOffset => {
                    // Millisecond position (later updated to be the remainder)
                    // We round up using the microsecond position
                    let duration    = frame_duration.get().as_micros();
                    let micros      = current_time.get().as_micros();
                    let millis      = if (micros%1000) >= 500 { (micros/1000) + 1 } else { micros/1000 };

                    // Compute minutes, seconds and the frame
                    let minutes     = millis / (60 * 1000);
                    let millis      = millis - (minutes * 60 * 1000);
                    let seconds     = millis / 1000;

                    let micros      = micros % 1_000_000;
                    let fps         = 1_000_000 / duration;
                    let frame       = micros / duration;
                    let frame       = (frame % fps)+1;

                    format!("T+{}:{:02}.{:02}", minutes, seconds, frame)
                }

                FrameDisplayStyle::FrameNumber => {
                    // Time and duration
                    let micros      = current_time.get().as_micros();
                    let duration    = frame_duration.get().as_micros();

                    let frame       = micros/duration + 1;

                    format!("F {}", frame)
                }
            }
        }))
    }

    ///
    /// Creates the UI for this controller
    ///
    fn ui(images: Arc<ResourceManager<Image>>, frame_text: BindRef<String>) -> BindRef<Control> {
        let frame_controls = images.get_named_resource("frame_controls");

        let ui = computed(move || {
            Control::container()
                .with(frame_controls.clone())
                .with(vec![
                    Control::empty()
                        .with(ControlAttribute::Padding((9, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),
                    Control::empty()
                        .with(ControlAttribute::Padding((9, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),
                    Control::empty()
                        .with(ControlAttribute::Padding((4, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),
                    Control::empty()
                        .with(ControlAttribute::Padding((4, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),
                    Control::empty()
                        .with(ControlAttribute::Padding((4, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),
                    Control::empty()
                        .with(ControlAttribute::Padding((4, 4), (4, 4)))
                        .with(Bounds::next_horiz(22.0)),

                    Control::empty()
                        .with(Bounds::next_horiz(4.0)),
                    Control::label()
                        .with(frame_text.get())
                        .with(TextAlign::Left)
                        .with(Font::Size(11.0))
                        .with(Font::Weight(FontWeight::Normal))
                        .with(ControlAttribute::Padding((4, 4), (9, 4)))
                        .with((ActionTrigger::Click, "ToggleTimeDisplay"))
                        .with(Bounds::next_horiz(76.0))
                ])
                .with(Bounds::next_horiz(22.0*6.0+80.0))
        });

        BindRef::new(&ui)
    }

    ///
    /// Creates the image resource manager for this controller
    ///
    fn images() -> ResourceManager<Image> {
        let images              = ResourceManager::new();

        let frame_controls      = images.register(svg_static(include_bytes!("../../svg/keyframes/frame_controls.svg")));

        images.assign_name(&frame_controls,     "frame_controls");

        images
    }
}

impl<Anim: 'static+Animation+EditableAnimation> Controller for FrameControlsController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "ToggleTimeDisplay" => {
                // Switch to the other frame style when the timer is clicked
                let new_style = match self.frame_style.get() {
                    FrameDisplayStyle::TimeOffset   => FrameDisplayStyle::FrameNumber,
                    FrameDisplayStyle::FrameNumber  => FrameDisplayStyle::TimeOffset
                };
                self.frame_style.set(new_style);
            }

            _ => { }
        }
    }
}
