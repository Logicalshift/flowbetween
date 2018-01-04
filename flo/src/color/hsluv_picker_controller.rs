use super::images::*;

use ui::*;
use canvas::*;
use binding::*;

use std::sync::*;

///
/// Controller that provides a colour picker using the HSLUV format
/// 
pub struct HsluvPickerController {
    ui:         BindRef<Control>,
    images:     Arc<ResourceManager<Image>>,
    canvases:   Arc<ResourceManager<BindingCanvas>>,

    viewmodel:  Arc<DynamicViewModel>,
    color:      Binding<Color>
}

impl HsluvPickerController {
    ///
    /// Creates a new HSLUV colour picker controller
    /// 
    pub fn new(color: &Binding<Color>) -> HsluvPickerController {
        let images      = ResourceManager::new();
        let canvases    = ResourceManager::new();
        let color       = color.clone();
        let viewmodel   = DynamicViewModel::new();

        // Viewmodel contains the H, S, L values
        let col = color.clone();
        viewmodel.set_computed("H", move || {
            PropertyValue::Float(col.get().to_hsluv_components().0 as f64)
        });
        let col = color.clone();
        viewmodel.set_computed("S", move || {
            PropertyValue::Float(col.get().to_hsluv_components().1 as f64)
        });
        let col = color.clone();
        viewmodel.set_computed("L", move || {
            PropertyValue::Float(col.get().to_hsluv_components().2 as f64)
        });

        // Set up the images
        let hsluv_wheel = HSLUV_COLOR_WHEEL.clone();
        let hsluv_wheel = images.register(hsluv_wheel);
        images.assign_name(&hsluv_wheel, "Wheel");

        let color_preview = Self::create_color_preview_canvas(&color);
        let color_preview = canvases.register(color_preview);

        // Set up the UI
        let ui          = Self::create_ui(&hsluv_wheel, &color_preview);
        
        // Controller is ready to go
        HsluvPickerController {
            ui:         ui,
            images:     Arc::new(images),
            canvases:   Arc::new(canvases),
            viewmodel:  Arc::new(viewmodel),
            color:      color
        }
    }

    ///
    /// Creates the preview canvas
    /// 
    fn create_color_preview_canvas(color: &Binding<Color>) -> BindingCanvas {
        let color = color.clone();

        // Just a circle with the colour in the center
        BindingCanvas::with_drawing(move |gc| {
            let color = color.get();

            // Colour preview
            gc.stroke_color(Color::Rgba(0.8, 0.9, 1.0, 0.8));
            gc.fill_color(color);
            gc.line_width(0.04);
            gc.circle(0.0, 0.0, 1.0-0.02);
            gc.fill();
            gc.stroke();

            // Hue indicator arrow
            let (hue, _, _, _)  = color.to_hsluv_components();
            let indicator_color = Color::Hsluv(hue, 100.0, 65.0, 1.0);

            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 0.7));
            gc.fill_color(indicator_color);
            gc.line_width_pixels(0.5);

            gc.new_path();
            gc.move_to(0.0, -0.99);
            gc.line_to(0.1, -0.80);
            gc.line_to(-0.1, -0.80);
            gc.close_path();
            gc.fill();
            gc.stroke();
        })
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn create_ui(hsluv_wheel: &Resource<Image>, preview: &Resource<BindingCanvas>) -> BindRef<Control> {
        // Constants
        let wheel_size      = 200.0;
        let preview_size    = f32::floor((140.0/256.0) * wheel_size)+2.0;

        // Bindings and images
        let hsluv_wheel = hsluv_wheel.clone();
        let preview     = preview.clone();

        BindRef::from(computed(move || {
            // The hue selector is designed to be cropped at the top of the screen
            let hue_selector = Control::rotor()
                .with(Bounds { 
                    x1: Position::At(0.0), 
                    y1: Position::At(-wheel_size/2.0), 
                    x2: Position::At(wheel_size), 
                    y2: Position::At(wheel_size/2.0)
                })
                .with(hsluv_wheel.clone())
                .with(State::Range((0.0.to_property(), 360.0.to_property())))
                .with(State::Value(Property::Bind("H".to_string())))
                .with((ActionTrigger::EditValue, "SetHue"))
                .with((ActionTrigger::SetValue, "SetHue"));
            
            // The preview control is in the same container as the hue selector
            let preview_control = Control::canvas()
                .with(preview.clone())
                .with(Bounds {
                    x1: Position::At((wheel_size-preview_size)/2.0),
                    y1: Position::At(-preview_size/2.0),
                    x2: Position::At((wheel_size+preview_size)/2.0),
                    y2: Position::At(preview_size/2.0)
                });
            
            // LHS is the luminance control
            let lhs = vec![
                Control::slider()
                    .with(Bounds::next_vert(24.0))
                    .with(State::Range((0.0.to_property(), 100.0.to_property())))
                    .with(State::Value(Property::Bind("L".to_string())))
                    .with((ActionTrigger::EditValue, "SetLum"))
                    .with((ActionTrigger::SetValue, "SetLum"))
            ];

            // RHS is the saturation control
            let rhs = vec![
                Control::slider()
                    .with(Bounds::next_vert(24.0))
                    .with(State::Range((0.0.to_property(), 100.0.to_property())))
                    .with(State::Value(Property::Bind("S".to_string())))
                    .with((ActionTrigger::EditValue, "SetSat"))
                    .with((ActionTrigger::SetValue, "SetSat"))
            ];

            // Put together the final colour selector
            let color_selector = Control::container()
                .with(Bounds::next_vert(wheel_size/2.0))
                .with(vec![
                    // LHS controls
                    Control::container()
                        .with(Bounds::stretch_horiz(1.0))
                        .with(ControlAttribute::Padding((0, 0), (8, 0)))
                        .with(lhs),
                    
                    // Main colour wheel
                    Control::cropping_container()
                        .with(Bounds::next_horiz(wheel_size))
                        .with(vec![
                            hue_selector,
                            preview_control
                        ]),

                    // RHS controls
                    Control::container()
                        .with(Bounds::stretch_horiz(1.0))
                        .with(ControlAttribute::Padding((8, 0), (0, 0)))
                        .with(rhs)
                ]);

            // Lay out the control
            Control::container()
                .with(vec![
                    color_selector
                ])
                .with(Bounds::fill_all())
        }))
    }
}

impl Controller for HsluvPickerController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Option<Arc<ViewModel>> {
        Some(self.viewmodel.clone())
    }

    fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> { None }

    fn action(&self, action_id: &str, action_data: &ActionParameter) {
        use ui::ActionParameter::*;
        use ui::PropertyValue::*;

        match (action_id, action_data) {
            ("SetHue", &Value(Float(new_hue))) => {
                let (_, s, l, a) = self.color.get().to_hsluv_components();
                self.color.clone().set(Color::Hsluv(new_hue as f32, s, l, a));
            },

            ("SetSat", &Value(Float(new_sat))) => {
                let (h, _, l, a) = self.color.get().to_hsluv_components();
                self.color.clone().set(Color::Hsluv(h, new_sat as f32, l, a));
            },

            ("SetLum", &Value(Float(new_lum))) => {
                let (h, s, _, a) = self.color.get().to_hsluv_components();
                self.color.clone().set(Color::Hsluv(h, s, new_lum as f32, a));
            },

            _ => ()
        }
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { 
        Some(Arc::clone(&self.images))
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(Arc::clone(&self.canvases))
    }
}
