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

    viewmodel:  Arc<DynamicViewModel>
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

        // Set up the images
        let hsluv_wheel = HSLUV_COLOR_WHEEL.clone();
        let hsluv_wheel = images.register(hsluv_wheel);
        images.assign_name(&hsluv_wheel, "Wheel");

        let color_preview = Self::create_color_preview_canvas(&color);
        let color_preview = canvases.register(color_preview);

        // Set up the UI
        let ui          = Self::create_ui(&color, &hsluv_wheel, &color_preview);
        
        // Controller is ready to go
        HsluvPickerController {
            ui:         ui,
            images:     Arc::new(images),
            canvases:   Arc::new(canvases),
            viewmodel:  Arc::new(viewmodel)
        }
    }

    ///
    /// Creates the preview canvas
    /// 
    fn create_color_preview_canvas(color: &Binding<Color>) -> BindingCanvas {
        let color = color.clone();

        // Just a circle with the colour in the center
        BindingCanvas::with_drawing(move |gc| {
            gc.stroke_color(Color::Rgba(0.8, 0.9, 1.0, 0.8));
            gc.line_width(0.04);
            gc.fill_color(color.get());
            gc.circle(0.0, 0.0, 1.0-0.02);
            gc.fill();
            gc.stroke();
        })
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn create_ui(color: &Binding<Color>, hsluv_wheel: &Resource<Image>, preview: &Resource<BindingCanvas>) -> BindRef<Control> {
        // Constants
        let wheel_size      = 200.0;
        let preview_size    = f32::floor((140.0/256.0) * wheel_size)+2.0;

        // Bindings and images
        let color       = color.clone();
        let hsluv_wheel = hsluv_wheel.clone();
        let preview     = preview.clone();

        BindRef::from(computed(move || {
            // The hue selector is designed to be cropped at the top of the screen
            let hue_selector = Control::empty()
                .with(Bounds { 
                    x1: Position::At(0.0), 
                    y1: Position::At(-wheel_size/2.0), 
                    x2: Position::At(wheel_size), 
                    y2: Position::At(wheel_size/2.0)
                })
                .with(hsluv_wheel.clone());
            
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
            ];

            // RHS is the saturation control
            let rhs = vec![
                Control::slider()
                    .with(Bounds::next_vert(24.0))
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

    fn action(&self, _action_id: &str, _action_data: &ActionParameter) {

    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { 
        Some(Arc::clone(&self.images))
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(Arc::clone(&self.canvases))
    }
}
