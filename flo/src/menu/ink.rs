use super::controls;
use super::super::color::*;
use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::brushes::*;

use std::f32;
use std::sync::*;

///
/// Controller used for the ink tool
///
pub struct InkMenuController {
    size:               Binding<f32>,
    opacity:            Binding<f32>,
    modification_mode:  Binding<BrushModificationMode>,
    representation:     Binding<BrushRepresentation>,
    brush_panel_open:   Binding<bool>,

    canvases:           Arc<ResourceManager<BindingCanvas>>,
    images:             Arc<ResourceManager<Image>>,
    ui:                 BindRef<Control>,
    view_model:         Arc<DynamicViewModel>,

    color_picker_open:  Binding<bool>,
    color_picker:       Arc<PopupController<ColorPickerController>>
}

impl InkMenuController {
    ///
    /// The image resources for the ink menu controller
    ///
    fn images() -> ResourceManager<Image> {
        let images = ResourceManager::new();

        let brush_settings_panel    = images.register(svg_static(include_bytes!("../../svg/menu_controls/brush_settings_x2.svg")));
        let active_settings_panel   = images.register(svg_static(include_bytes!("../../svg/menu_controls/active_brush_settings_x2.svg")));
        let additive_mode           = images.register(svg_static(include_bytes!("../../svg/brush_modes/additive.svg")));
        let individual_mode         = images.register(svg_static(include_bytes!("../../svg/brush_modes/individual.svg")));
        let path_editing            = images.register(svg_static(include_bytes!("../../svg/brush_modes/path_editing.svg")));
        let brush_stroke            = images.register(svg_static(include_bytes!("../../svg/brush_modes/brush_stroke.svg")));

        let combo_picker            = images.register(svg_static(include_bytes!("../../svg/control_decals/combo_picker.svg")));
        let settings_cog            = images.register(svg_static(include_bytes!("../../svg/control_decals/settings_cog.svg")));

        images.assign_name(&brush_settings_panel,   "brush_settings");
        images.assign_name(&active_settings_panel,  "active_settings");
        images.assign_name(&additive_mode,          "additive_mode");
        images.assign_name(&individual_mode,        "individual_mode");
        images.assign_name(&path_editing,           "path_editing");
        images.assign_name(&brush_stroke,           "brush_stroke");

        images.assign_name(&combo_picker,           "combo_picker");
        images.assign_name(&settings_cog,           "settings_cog");

        images
    }

    ///
    /// Creates a new ink menu controller
    ///
    pub fn new(size: &Binding<f32>, opacity: &Binding<f32>, colour: &Binding<Color>, modification_mode: &Binding<BrushModificationMode>, representation: &Binding<BrushRepresentation>) -> InkMenuController {
        // Set up the view model
        let view_model = Arc::new(DynamicViewModel::new());

        let vm_size                 = size.clone();
        let vm_opacity              = opacity.clone();
        let brush_panel_open        = bind(false);

        view_model.set_computed("Size", move || PropertyValue::Float(vm_size.get() as f64));
        view_model.set_computed("Opacity", move || PropertyValue::Float(vm_opacity.get() as f64));

        view_model.set_property("EditSize", PropertyValue::Bool(false));
        view_model.set_property("EditOpacity", PropertyValue::Bool(false));
        let edit_brush_properties   = brush_panel_open.clone();
        view_model.set_computed("EditBrushProperties", move || PropertyValue::Bool(edit_brush_properties.get()));

        // Create the colour picker popup
        let color_picker_open   = Binding::new(false);
        let color_picker        = ColorPickerController::new(colour);
        let color_picker        = PopupController::new(color_picker, &color_picker_open)
            .with_direction(&PopupDirection::Below)
            .with_size(&(500, 124));

        let vm_color_picker_open = color_picker_open.clone();
        view_model.set_computed("ColorPickerOpen", move || PropertyValue::Bool(vm_color_picker_open.get()));

        // Images
        let images                      = Arc::new(Self::images());

        // Create the canvases
        let canvases                = Arc::new(ResourceManager::new());

        let brush_preview           = Self::brush_preview(size, opacity, colour);
        let brush_preview           = canvases.register(brush_preview);
        canvases.assign_name(&brush_preview, "BrushPreview");

        let size_preview            = Self::size_preview(size, 32.0 - 6.0);
        let size_preview            = canvases.register(size_preview);
        canvases.assign_name(&size_preview, "SizePreview");

        let size_preview_large      = Self::size_preview(size, 100.0);
        let size_preview_large      = canvases.register(size_preview_large);
        canvases.assign_name(&size_preview_large, "SizePreview2");

        let opacity_preview         = Self::opacity_preview(opacity, 32.0-6.0);
        let opacity_preview         = canvases.register(opacity_preview);
        canvases.assign_name(&opacity_preview, "OpacityPreview");

        let opacity_preview_large   = Self::opacity_preview(opacity, 84.0);
        let opacity_preview_large   = canvases.register(opacity_preview_large);
        canvases.assign_name(&opacity_preview_large, "OpacityPreview2");

        let colour_preview          = Self::colour_preview(colour);
        let colour_preview          = canvases.register(colour_preview);
        canvases.assign_name(&colour_preview, "ColourPreview");

        // Generate the UI
        let ui = Self::ui(&canvases, &images, &brush_panel_open, modification_mode, representation);

        // Finalize the control
        InkMenuController {
            size:               size.clone(),
            opacity:            opacity.clone(),
            modification_mode:  modification_mode.clone(),
            representation:     representation.clone(),
            brush_panel_open:   brush_panel_open,

            canvases:           canvases,
            images:             images,
            ui:                 ui,
            view_model:         view_model,

            color_picker_open:  color_picker_open,
            color_picker:       Arc::new(color_picker)
        }
    }

    ///
    /// Creates the UI for the ink menu bar
    ///
    fn ui(canvases: &ResourceManager<BindingCanvas>, images: &ResourceManager<Image>, brush_panel_open: &Binding<bool>, modification_mode: &Binding<BrushModificationMode>, representation: &Binding<BrushRepresentation>) -> BindRef<Control> {
        // Model
        let modification_mode           = modification_mode.clone();
        let representation              = representation.clone();
        let brush_panel_open            = brush_panel_open.clone();

        // Fetch the image resources
        let brush_settings_background   = images.get_named_resource("brush_settings");
        let active_settings_background  = images.get_named_resource("active_settings");
        let additive_mode               = images.get_named_resource("additive_mode");
        let individual_mode             = images.get_named_resource("individual_mode");
        let path_editing_mode           = images.get_named_resource("path_editing");
        let brush_stroke_mode           = images.get_named_resource("brush_stroke");

        let combo_picker                = images.get_named_resource("combo_picker");
        let settings_cog                = images.get_named_resource("settings_cog");

        // ... and the canvas resources
        let brush_preview               = canvases.get_named_resource("BrushPreview");
        let size_preview                = canvases.get_named_resource("SizePreview");
        let size_preview_large          = canvases.get_named_resource("SizePreview2");
        let opacity_preview             = canvases.get_named_resource("OpacityPreview");
        let opacity_preview_large       = canvases.get_named_resource("OpacityPreview2");
        let colour_preview              = canvases.get_named_resource("ColourPreview");

        // Generate the UI control
        let ui = computed(move || {
            let modification_mode   = modification_mode.get();
            let representation      = representation.get();
            let brush_panel_open    = brush_panel_open.get();

            let modification_icon   = match modification_mode {
                BrushModificationMode::Additive     => additive_mode.clone(),
                BrushModificationMode::Individual   => individual_mode.clone()
            };
            let representation_icon = match representation {
                BrushRepresentation::BrushStroke    => brush_stroke_mode.clone(),
                BrushRepresentation::Path           => path_editing_mode.clone()
            };
            let brush_settings_background = if brush_panel_open {
                &active_settings_background
            } else {
                &brush_settings_background
            };

            let modification_text   = match modification_mode {
                BrushModificationMode::Additive     => "Combine paths into one",
                BrushModificationMode::Individual   => "Create separate paths"
            };
            let representation_text = match representation {
                BrushRepresentation::BrushStroke    => "Keep brush strokes",
                BrushRepresentation::Path           => "Convert to paths"
            };

            Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((0, 3), (0, 3)))
                .with(vec![
                    controls::divider(),

                    Control::label()
                        .with("Brush:")
                        .with(FontWeight::Light)
                        .with(TextAlign::Right)
                        .with(Font::Size(14.0))
                        .with(Bounds::next_horiz(48.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                    Control::canvas()
                        .with(brush_preview.clone())
                        .with(Bounds::next_horiz(64.0)),

                    controls::divider(),

                    Control::label()
                        .with("Color:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(40.0)),
                    Control::empty().with(Bounds::next_horiz(4.0)),
                    Control::canvas()
                        .with(colour_preview.clone())
                        .with(Bounds::next_horiz(32.0))
                        .with(State::Badged(Property::Bind("ColorPickerOpen".to_string())))
                        .with((ActionTrigger::Click, "ShowColorPopup"))
                        .with_controller("ColorPopup"),

                    controls::divider(),

                    Control::label()
                        .with("Size:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(36.0)),
                    Control::empty().with(Bounds::next_horiz(6.0)),
                    Control::slider()
                        .with(State::Range((0.0.to_property(), 50.0.to_property())))
                        .with(State::Value(Property::Bind("Size".to_string())))
                        .with(Bounds::next_horiz(96.0))
                        .with((ActionTrigger::EditValue, "ChangeSizeEdit".to_string()))
                        .with((ActionTrigger::SetValue, "ChangeSizeSet".to_string())),
                    Control::empty().with(Bounds::next_horiz(4.0)),
                    Control::canvas()
                        .with(size_preview.clone())
                        .with(Bounds::next_horiz(32.0))
                        .with(vec![
                            Control::popup()
                                .with(Popup::IsOpen(Property::Bind("EditSize".to_string())))
                                .with(Popup::Direction(PopupDirection::Below))
                                .with(Popup::Size(100, 100))
                                .with(Popup::Offset(14))
                                .with(ControlAttribute::ZIndex(1000))
                                .with(vec![
                                    Control::canvas()
                                        .with(Bounds::fill_all())
                                        .with(size_preview_large.clone())
                                ])
                        ]),

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
                        ]),

                    controls::divider(),

                    Control::empty()
                        .with(Bounds::next_horiz(4.0)),
                    Control::container()
                        .with(brush_settings_background.clone())
                        .with(Bounds::next_horiz(92.0))
                        .with(ControlAttribute::Padding((3, 3), (10, 3)))
                        .with(vec![
                            Control::empty()
                                .with(Bounds::next_horiz(35.0))
                                .with(if !brush_panel_open { (ActionTrigger::Click, "ShowBrushPropertiesPopup") } else { (ActionTrigger::Click, "HideBrushPropertiesPopup") })
                                .with(if brush_panel_open { vec![
                                    Control::popup()
                                        .with(Popup::Direction(PopupDirection::Below))
                                        .with(Popup::Size(220, 131))
                                        .with(Popup::Offset(14))
                                        .with(ControlAttribute::ZIndex(1000))
                                        .with(Popup::IsOpen(Property::Bind("EditBrushProperties".to_string())))
                                        .with((ActionTrigger::Dismiss, "HideBrushPropertiesPopup"))
                                        .with(vec![
                                            Control::container()
                                                .with(Bounds::fill_all())
                                                .with(ControlAttribute::Padding((10, 0), (10, 0)))
                                                .with(vec![
                                                    Control::empty()
                                                        .with(Bounds::next_vert(3.0)),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(30.0))
                                                        .with(settings_cog.clone()),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(3.0)),
                                                    Control::label()
                                                        .with(Bounds::next_vert(26.0))
                                                        .with("Settings")
                                                        .with(Font::Size(15.0))
                                                        .with(Font::Align(TextAlign::Center))
                                                        .with(Font::Weight(FontWeight::Light)),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(1.0)),
                                                    Control::empty()
                                                        .with(Appearance::Background(MENU_BACKGROUND_ALT))
                                                        .with(Bounds::next_vert(2.0)),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(3.0)),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(26.0))
                                                        .with(combo_picker.clone())
                                                        .with((ActionTrigger::Click, "NextModificationMode"))
                                                        .with(ControlAttribute::Padding((24, 4), (24, 4)))
                                                        .with(vec![
                                                            Control::empty()
                                                                .with(Bounds::next_horiz(20.0))
                                                                .with(modification_icon.clone()),
                                                            Control::empty()
                                                                .with(Bounds::next_horiz(4.0)),
                                                            Control::label()
                                                                .with(Bounds::fill_horiz())
                                                                .with(Font::Size(11.0))
                                                                .with(modification_text)
                                                        ]),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(3.0)),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(26.0))
                                                        .with(combo_picker.clone())
                                                        .with((ActionTrigger::Click, "NextBrushRepresentation"))
                                                        .with(ControlAttribute::Padding((24, 4), (24, 4)))
                                                        .with(vec![
                                                            Control::empty()
                                                                .with(Bounds::next_horiz(20.0))
                                                                .with(representation_icon.clone()),
                                                            Control::empty()
                                                                .with(Bounds::next_horiz(4.0)),
                                                            Control::label()
                                                                .with(Bounds::fill_horiz())
                                                                .with(Font::Size(11.0))
                                                                .with(representation_text)
                                                        ]),
                                                    Control::empty()
                                                        .with(Bounds::next_vert(3.0)),
                                                ])
                                        ]),
                                ] } else { vec![] }),
                            Control::empty()
                                .with(Bounds::next_horiz(20.0))
                                .with(modification_icon)
                                .with((ActionTrigger::Click, "NextModificationMode")),
                            Control::empty()
                                .with(Bounds::next_horiz(4.0)),
                            Control::empty()
                                .with(Bounds::next_horiz(20.0))
                                .with(representation_icon)
                                .with((ActionTrigger::Click, "NextBrushRepresentation"))
                        ]),

                ])
        });

        BindRef::from(ui)
    }

    ///
    /// Creates the size preview canvas
    ///
    pub fn size_preview(size: &Binding<f32>, control_height: f32) -> BindingCanvas {
        let size            = size.clone();

        BindingCanvas::with_drawing(move |gc| {
            let size = size.get();

            gc.canvas_height(control_height);
            gc.fill_color(Color::Rgba(0.8, 0.8, 0.8, 1.0));

            gc.new_path();
            gc.circle(0.0, 0.0, size/2.0);
            gc.fill();
        })
    }

    ///
    /// Creates the opacity preview canvas
    ///
    pub fn opacity_preview(opacity: &Binding<f32>, control_height: f32) -> BindingCanvas {
        let opacity         = opacity.clone();

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
    pub fn colour_preview(colour: &Binding<Color>) -> BindingCanvas {
        let colour          = colour.clone();
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
    /// Creates the brush preview canvas
    ///
    pub fn brush_preview(size: &Binding<f32>, opacity: &Binding<f32>, color: &Binding<Color>) -> BindingCanvas {
        let size    = size.clone();
        let opacity = opacity.clone();
        let color   = color.clone();

        let control_height  = 32.0 - 6.0;
        let control_width   = 64.0;
        let preview_width   = control_width - 8.0;
        let preview_height  = control_height - 12.0;

        BindingCanvas::with_drawing(move |gc| {
            // Canvas height should match the control height
            gc.canvas_height(control_height);
            gc.center_region(-control_width/2.0, -control_height/2.0, control_width/2.0, control_height/2.0);

            // Clear the background
            gc.fill_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));
            gc.rect(-control_width/2.0, -control_height/2.0, control_width/2.0, control_height/2.0);
            gc.fill();

            // Create an ink brush
            let brush = InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw);

            // Render a test brush stroke
            let mut points = vec![];
            for point in 0..100 {
                let point   = (point as f32)/100.0;
                let offset  = -(point*f32::consts::PI*1.5).cos();

                points.push(RawPoint {
                    position:   (point*preview_width-(preview_width/2.0), offset*preview_height/2.0),
                    tilt:       (0.0, 0.0),
                    pressure:   point
                })
            }

            // Create the properties
            let brush_properties = BrushProperties {
                size:       size.get(),
                opacity:    opacity.get(),
                color:      color.get()
            };

            let points = brush.brush_points_for_raw_points(&points);

            gc.draw_list(brush.prepare_to_render(&brush_properties));
            gc.draw_list(brush.render_brush(&brush_properties, &points, Arc::new(vec![])));
        })
    }
}

impl Controller for InkMenuController {
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
        Some(Arc::clone(&self.canvases))
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use self::ActionParameter::*;

        match (action_id, action_parameter) {
            ("ChangeSizeEdit", &Value(PropertyValue::Float(new_size))) => {
                // User has dragged the 'size' property
                self.size.set(new_size as f32);
                self.view_model.set_property("EditSize", PropertyValue::Bool(true));
            },

            ("ChangeSizeSet", &Value(PropertyValue::Float(new_size))) => {
                // User has dragged the 'size' property
                self.size.set(new_size as f32);
                self.view_model.set_property("EditSize", PropertyValue::Bool(false));
            },

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
            },

            ("ShowBrushPropertiesPopup", _) => {
                // User has clicked the brush properties icon
                self.brush_panel_open.set(true);
            },

            ("HideBrushPropertiesPopup", _) => {
                // User has dismissed the brush properties dialog
                self.brush_panel_open.set(false);
            },

            ("NextModificationMode", _) => {
                self.modification_mode.set(match self.modification_mode.get() {
                    BrushModificationMode::Additive     => BrushModificationMode::Individual,
                    BrushModificationMode::Individual   => BrushModificationMode::Additive
                });
            },

            ("NextBrushRepresentation", _) => {
                self.representation.set(match self.representation.get() {
                    BrushRepresentation::Path           => BrushRepresentation::BrushStroke,
                    BrushRepresentation::BrushStroke    => BrushRepresentation::Path
                });
            },

            _ => ()
        }
    }
}
