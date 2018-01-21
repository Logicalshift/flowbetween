use super::super::style::*;
use super::super::color::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;
use animation::brushes::*;

use std::f32;
use std::sync::*;

pub const INKMENUCONTROLLER: &str = "InkMenu";

///
/// Controller used for the ink tool
/// 
pub struct InkMenuController {
    size:               Binding<f32>,
    opacity:            Binding<f32>,

    canvases:           Arc<ResourceManager<BindingCanvas>>,
    ui:                 BindRef<Control>,
    view_model:         Arc<DynamicViewModel>,

    color_picker_open:  Binding<bool>,
    color_picker:       Arc<PopupController<ColorPickerController>>
}

impl InkMenuController {
    ///
    /// Creates a new ink menu controller
    /// 
    pub fn new(size: &Binding<f32>, opacity: &Binding<f32>, colour: &Binding<Color>) -> InkMenuController {
        // Set up the view model
        let view_model = Arc::new(DynamicViewModel::new());

        let vm_size     = size.clone();
        let vm_opacity  = opacity.clone();

        view_model.set_computed("Size", move || PropertyValue::Float(vm_size.get() as f64));
        view_model.set_computed("Opacity", move || PropertyValue::Float(vm_opacity.get() as f64));

        view_model.set_property("EditSize", PropertyValue::Bool(false));
        view_model.set_property("EditOpacity", PropertyValue::Bool(false));

        // Create the colour picker popup
        let color_picker_open   = Binding::new(false);
        let color_picker        = ColorPickerController::new(colour);
        let color_picker        = PopupController::new(color_picker, &color_picker_open)
            .with_direction(&PopupDirection::Below)
            .with_size(&(500, 124));

        let vm_color_picker_open = color_picker_open.clone();
        view_model.set_computed("ColorPickerOpen", move || PropertyValue::Bool(vm_color_picker_open.get()));

        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

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
        let ui = BindRef::from(bind(Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((0, 3), (0, 3)))
                .with(vec![
                    Self::divider(),

                    Control::label()
                        .with("Brush:")
                        .with(FontWeight::Light)
                        .with(TextAlign::Right)
                        .with(Font::Size(14.0))
                        .with(Bounds::next_horiz(48.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                    Control::canvas()
                        .with(brush_preview)
                        .with(Bounds::next_horiz(64.0)),

                    Self::divider(),

                    Control::label()
                        .with("Color:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(40.0)),
                    Control::empty().with(Bounds::next_horiz(4.0)),
                    Control::canvas()
                        .with(colour_preview)
                        .with(Bounds::next_horiz(32.0))
                        .with(State::Badged(Property::Bind("ColorPickerOpen".to_string())))
                        .with((ActionTrigger::Click, "ShowColorPopup"))
                        .with_controller("ColorPopup"),

                    Self::divider(),

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
                        .with(size_preview)
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
                                        .with(size_preview_large)
                                ])
                        ]),

                    Self::divider(),

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
                        .with(opacity_preview)
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
                                        .with(opacity_preview_large)
                                ])
                        ]),

                    Control::empty()
                        .with(Bounds::next_horiz(16.0)),
                    Self::divider()
                ])));

        // Finalize the control
        InkMenuController {
            size:               size.clone(),
            opacity:            opacity.clone(),

            canvases:           canvases, 
            ui:                 ui,
            view_model:         view_model,

            color_picker_open:  color_picker_open,
            color_picker:       Arc::new(color_picker)
        }
    }

    pub fn divider() -> Control {
        Control::container()
            .with(vec![
                Control::empty()
                    .with(Bounds::next_horiz(5.0)),
                Control::empty()
                    .with(Bounds::next_horiz(2.0))
                    .with(Appearance::Background(MENU_BACKGROUND_ALT)),
                Control::empty()
                    .with(Bounds::next_horiz(5.0)),
            ])
            .with(Bounds::next_horiz(12.0))
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

            brush.prepare_to_render(gc, &brush_properties);
            brush.render_brush(gc, &brush_properties, &points);
        })
    }
}

impl Controller for InkMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Option<Arc<ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
        match id {
            "ColorPopup"        => Some(self.color_picker.clone()),
            _                   => None
        }
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(self.canvases.clone())
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            ("ChangeSizeEdit", &Value(PropertyValue::Float(new_size))) => {
                // User has dragged the 'size' property
                self.size.clone().set(new_size as f32);
                self.view_model.set_property("EditSize", PropertyValue::Bool(true));
            },

            ("ChangeSizeSet", &Value(PropertyValue::Float(new_size))) => {
                // User has dragged the 'size' property
                self.size.clone().set(new_size as f32);
                self.view_model.set_property("EditSize", PropertyValue::Bool(false));
            },

            ("ChangeOpacityEdit", &Value(PropertyValue::Float(new_opacity))) => {
                // User has dragged the 'opacity' property
                self.opacity.clone().set(new_opacity as f32);
                self.view_model.set_property("EditOpacity", PropertyValue::Bool(true));
            },

            ("ChangeOpacitySet", &Value(PropertyValue::Float(new_opacity))) => {
                // User has dragged the 'opacity' property
                self.opacity.clone().set(new_opacity as f32);
                self.view_model.set_property("EditOpacity", PropertyValue::Bool(false));
            },

            ("ShowColorPopup", _) => {
                // User has clicked the colour icon
                self.color_picker_open.clone().set(true)
            }

            _ => ()
        }
    }
}
