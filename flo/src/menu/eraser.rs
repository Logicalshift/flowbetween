use super::ink::*;
use super::controls;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::brushes::*;

use std::f32;
use std::sync::*;

///
/// Controller used for the eraser tool
///
pub struct EraserMenuController {
    size:               Binding<f32>,
    opacity:            Binding<f32>,

    canvases:           Arc<ResourceManager<BindingCanvas>>,
    ui:                 BindRef<Control>,
    view_model:         Arc<DynamicViewModel>,
}

impl EraserMenuController {
    ///
    /// Creates a new eraser menu controller
    ///
    pub fn new(size: &Binding<f32>, opacity: &Binding<f32>) -> EraserMenuController {
        // Set up the view model
        let view_model = Arc::new(DynamicViewModel::new());

        let vm_size     = size.clone();
        let vm_opacity  = opacity.clone();

        view_model.set_computed("Size", move || PropertyValue::Float(vm_size.get() as f64));
        view_model.set_computed("Opacity", move || PropertyValue::Float(vm_opacity.get() as f64));

        view_model.set_property("EditSize", PropertyValue::Bool(false));
        view_model.set_property("EditOpacity", PropertyValue::Bool(false));

        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        let brush_preview           = Self::eraser_preview(size, opacity);
        let brush_preview           = canvases.register(brush_preview);
        canvases.assign_name(&brush_preview, "BrushPreview");

        let size_preview            = InkMenuController::size_preview(size, 32.0 - 6.0);
        let size_preview            = canvases.register(size_preview);
        canvases.assign_name(&size_preview, "SizePreview");

        let size_preview_large      = InkMenuController::size_preview(size, 100.0);
        let size_preview_large      = canvases.register(size_preview_large);
        canvases.assign_name(&size_preview_large, "SizePreview2");

        let opacity_preview         = InkMenuController::opacity_preview(opacity, 32.0-6.0);
        let opacity_preview         = canvases.register(opacity_preview);
        canvases.assign_name(&opacity_preview, "OpacityPreview");

        let opacity_preview_large   = InkMenuController::opacity_preview(opacity, 84.0);
        let opacity_preview_large   = canvases.register(opacity_preview_large);
        canvases.assign_name(&opacity_preview_large, "OpacityPreview2");

        // Generate the UI
        let ui = BindRef::from(bind(Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((0, 3), (0, 3)))
                .with(vec![
                    controls::divider(),

                    Control::label()
                        .with("Eraser:")
                        .with(FontWeight::Light)
                        .with(TextAlign::Right)
                        .with(Font::Size(14.0))
                        .with(Bounds::next_horiz(48.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                    Control::canvas()
                        .with(brush_preview)
                        .with(Bounds::next_horiz(64.0)),

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
                    controls::divider()
                ])));

        // Finalize the control
        EraserMenuController {
            size:               size.clone(),
            opacity:            opacity.clone(),

            canvases:           canvases,
            ui:                 ui,
            view_model:         view_model,
        }
    }

    ///
    /// Creates the erasaer preview canvas
    ///
    pub fn eraser_preview(size: &Binding<f32>, opacity: &Binding<f32>) -> BindingCanvas {
        let size    = size.clone();
        let opacity = opacity.clone();

        let control_height  = 32.0 - 6.0;
        let control_width   = 64.0;
        let preview_width   = control_width - 8.0;
        let preview_height  = control_height - 12.0;

        BindingCanvas::with_drawing(move |gc| {
            // Canvas height should match the control height
            gc.canvas_height(control_height);
            gc.center_region(-control_width/2.0, -control_height/2.0, control_width/2.0, control_height/2.0);

            // Clear the background
            gc.layer(0);
            gc.fill_color(Color::Rgba(0.9, 0.9, 0.9, 1.0));
            gc.rect(-control_width/2.0, -control_height/2.0, control_width/2.0, control_height/2.0);
            gc.fill();

            gc.layer(1);
            gc.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            gc.rect(-control_width/2.0, -control_height/2.0, control_width/2.0, control_height/2.0);
            gc.fill();

            // Create an ink brush
            let brush = InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Erase);

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
                color:      Color::Rgba(0.0, 0.0, 0.0, 1.0)
            };

            let points = brush.brush_points_for_raw_points(&points);

            gc.draw_list(brush.prepare_to_render(&brush_properties));
            gc.draw_list(brush.render_brush(&brush_properties, &points, Arc::new(vec![])));
        })
    }
}

impl Controller for EraserMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        Some(self.canvases.clone())
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
            }

            _ => ()
        }
    }
}
