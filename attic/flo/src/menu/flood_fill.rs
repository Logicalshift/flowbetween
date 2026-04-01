use super::controls;
use super::super::color::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;

use std::sync::*;

///
/// Menu controller used for the flood fill tool
///
pub struct FloodFillMenuController {
    opacity:            Binding<f32>,

    canvases:           Arc<ResourceManager<BindingCanvas>>,
    ui:                 BindRef<Control>,
    view_model:         Arc<DynamicViewModel>,

    color_picker_open:  Binding<bool>,
    color_picker:       Arc<PopupController<ColorPickerController>>
}

impl FloodFillMenuController {
    ///
    /// Creates a new flood fill menu controller
    ///
    pub fn new(color: Binding<Color>, opacity: Binding<f32>) -> FloodFillMenuController {
        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        // Colour picker
        let color_picker_open       = bind(false);
        let color_picker            = ColorPickerController::new(&color);
        let color_picker            = PopupController::new(color_picker, &color_picker_open)
            .with_direction(&PopupDirection::Below)
            .with_size(&(500, 124));
        let color_picker            = Arc::new(color_picker);

        // Create the viewmodel
        let vm_opacity              = Binding::clone(&opacity);
        let vm_color_picker_open    = Binding::clone(&color_picker_open);
        let view_model              = Arc::new(DynamicViewModel::new());

        view_model.set_computed("Opacity", move || PropertyValue::Float(vm_opacity.get() as f64));
        view_model.set_property("EditOpacity", PropertyValue::Bool(false));
        view_model.set_computed("ColorPickerOpen", move || PropertyValue::Bool(vm_color_picker_open.get()));

        // Build the UI
        let ui = Self::ui(BindRef::from(color.clone()), BindRef::from(opacity.clone()), Arc::clone(&canvases));

        FloodFillMenuController {
            opacity:            opacity,

            canvases:           canvases,
            ui:                 ui,
            view_model:         view_model,

            color_picker_open:  color_picker_open,
            color_picker:       color_picker
        }
    }

    ///
    /// Creates the opacity preview canvas
    ///
    pub fn opacity_preview(opacity: BindRef<f32>, control_height: f32) -> BindingCanvas {
        BindingCanvas::with_drawing(move |gc| {
            let size = control_height - 8.0;

            gc.canvas_height(control_height);
            gc.line_width(2.0);
            gc.stroke_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));
            gc.fill_color(Color::Rgba(0.8, 0.8, 0.8, opacity.get()));

            gc.new_path();
            gc.circle(0.0, 0.0, size/2.0);
            gc.fill();
            gc.stroke();
        })
    }

    ///
    /// Creates the colour preview canvas
    ///
    pub fn color_preview(colour: BindRef<Color>) -> BindingCanvas {
        let control_height  = 32.0 - 6.0;

        BindingCanvas::with_drawing(move |gc| {
            let size = control_height - 8.0;

            gc.canvas_height(control_height);
            gc.line_width(2.0);
            gc.stroke_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));
            gc.fill_color(colour.get().with_alpha(1.0));

            gc.new_path();
            gc.circle(0.0, 0.0, size/2.0);
            gc.fill();
            gc.stroke();
        })
    }

    ///
    /// Creates the UI for this menu
    ///
    fn ui(color: BindRef<Color>, opacity: BindRef<f32>, canvases: Arc<ResourceManager<BindingCanvas>>) -> BindRef<Control> {
        // Create the canvases
        let color_preview           = Self::color_preview(color);
        let opacity_preview         = Self::opacity_preview(opacity.clone(), 32.0-6.0);
        let opacity_preview_large   = Self::opacity_preview(opacity, 100.0);

        let color_preview           = canvases.register(color_preview);
        let opacity_preview         = canvases.register(opacity_preview);
        let opacity_preview_large   = canvases.register(opacity_preview_large);

        // Generate the UI
        let ui = computed(move ||
            Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((0, 3), (0, 3)))
                .with(vec![
                    controls::divider(),

                    Control::label()
                        .with("Flood fill:")
                        .with(FontWeight::Light)
                        .with(TextAlign::Right)
                        .with(Font::Size(14.0))
                        .with(Bounds::next_horiz(64.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),

                    Control::label()
                        .with("Color:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(40.0)),
                    Control::empty().with(Bounds::next_horiz(4.0)),
                    Control::canvas()
                        .with(color_preview.clone())
                        .with(Bounds::next_horiz(32.0))
                        .with(State::Badged(Property::Bind("ColorPickerOpen".to_string())))
                        .with((ActionTrigger::Click, "ShowColorPopup"))
                        .with_controller("ColorPopup"),

                    controls::divider(),

                    Control::label()
                        .with("Opacity:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(56.0)),
                    Control::empty().with(Bounds::next_horiz(6.0)),
                    Control::slider()
                        .with(State::Range((0.0.to_property(), 1.0.to_property())))
                        .with(State::Value(Property::Bind("Opacity".to_string())))
                        .with(Bounds::next_horiz(96.0))
                        .with((ActionTrigger::EditValue, "ChangeOpacityEdit".to_string()))
                        .with((ActionTrigger::SetValue, "ChangeOpacitySet".to_string())),
                    Control::empty().with(Bounds::next_horiz(4.0)),
                    Control::canvas()
                        .with(opacity_preview.clone())
                        .with(Bounds::next_horiz(32.0))
                        .with(vec![
                            Control::popup()
                                .with(Popup::IsOpen(Property::Bind("EditOpacity".to_string())))
                                .with(Popup::Direction(PopupDirection::Below))
                                .with(Popup::Size(100, 100))
                                .with(Popup::Offset(14))
                                .with(ControlAttribute::ZIndex(1000))
                                .with(ControlAttribute::Padding((8, 8), (8, 8)))
                                .with(vec![
                                    Control::canvas()
                                        .with(Bounds::fill_all())
                                        .with(opacity_preview_large.clone())
                                ])
                        ])
                ])
            );

        BindRef::from(ui)
    }
}

impl Controller for FloodFillMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        match id {
            "ColorPopup"        => Some(self.color_picker.clone()),
            _                   => None
        }
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        Some(self.canvases.clone())
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use self::ActionParameter::*;

        match (action_id, action_parameter) {
            ("ChangeOpacityEdit", &Value(PropertyValue::Float(new_opacity))) => {
                // User has dragged the 'opacity' property
                self.opacity.set(new_opacity as f32);
                self.view_model.set_property("EditOpacity", PropertyValue::Bool(true));
            },

            ("ChangeOpacitySet", &Value(PropertyValue::Float(new_opacity))) => {
                // User has dragged the 'opacity' property
                self.opacity.set(new_opacity as f32);
                self.view_model.set_property("EditOpacity", PropertyValue::Bool(false));
            },

            ("ShowColorPopup", _) => {
                // User has clicked the colour icon
                self.color_picker_open.set(true)
            }

            _ => ()
        }
    }
}
