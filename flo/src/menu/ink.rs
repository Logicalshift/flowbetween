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
    size:       Binding<f32>,

    canvases:   Arc<ResourceManager<BindingCanvas>>,
    ui:         BindRef<Control>,
    view_model: Arc<DynamicViewModel>
}

impl InkMenuController {
    ///
    /// Creates a new ink menu controller
    /// 
    pub fn new(size: &Binding<f32>, opacity: &Binding<f32>, color: &Binding<Color>) -> InkMenuController {
        // Set up the view model
        let view_model = Arc::new(DynamicViewModel::new());

        let vm_size = size.clone();
        view_model.set_computed("Size", move || PropertyValue::Float(vm_size.get() as f64));

        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        let brush_preview   = Self::brush_preview(size, opacity, color);
        let brush_preview   = canvases.register(brush_preview);
        canvases.assign_name(&brush_preview, "BrushPreview");

        let size_preview    = Self::size_preview(size);
        let size_preview    = canvases.register(size_preview);
        canvases.assign_name(&size_preview, "SizePreview");

        // Generate the UI
        let ui = BindRef::from(bind(Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::label()
                        .with("Ink:")
                        .with(TextAlign::Right)
                        .with(Bounds::next_horiz(48.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),
                    Control::canvas()
                        .with(brush_preview)
                        .with(Bounds::next_horiz(64.0)),

                    Control::empty()
                        .with(Bounds::next_horiz(8.0)),

                    Control::canvas()
                        .with(size_preview)
                        .with(Bounds::next_horiz(32.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(4.0)),
                    Control::slider()
                        .with(State::Range((0.0.to_property(), 50.0.to_property())))
                        .with(State::Value(Property::Bind("Size".to_string())))
                        .with(Bounds::next_horiz(96.0))
                        .with((ActionTrigger::EditValue, "ChangeSize".to_string()))
                        .with((ActionTrigger::SetValue, "ChangeSize".to_string()))
                ])));

        // Finalize the control
        InkMenuController {
            size:       size.clone(),

            canvases:   canvases, 
            ui:         ui,
            view_model: view_model
        }
    }

    ///
    /// Creates the size preview canvas
    /// 
    pub fn size_preview(size: &Binding<f32>) -> BindingCanvas {
        let size            = size.clone();
        let control_height  = 32.0;

        BindingCanvas::with_drawing(move |gc| {
            let size = size.get();

            gc.canvas_height(control_height);
            gc.fill_color(Color::Rgba(0.8, 0.8, 0.8, 1.0));

            // TODO: make this a circle rather than a rect
            gc.new_path();
            gc.rect(-size/2.0, -size/2.0, size/2.0, size/2.0);
            gc.fill();
        })
    }

    ///
    /// Creates the brush preview canvas
    /// 
    pub fn brush_preview(_size: &Binding<f32>, _opacity: &Binding<f32>, _color: &Binding<Color>) -> BindingCanvas {
        let control_height  = 32.0;
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

                points.push(BrushPoint {
                    position: (point*preview_width-(preview_width/2.0), offset*preview_height/2.0),
                    pressure: point
                })
            }

            brush.prepare_to_render(gc);
            brush.render_brush(gc, &points);
        })
    }
}

impl Controller for InkMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { 
        Some(self.canvases.clone())
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use ui::ActionParameter::*;

        match (action_id, action_parameter) {
            ("ChangeSize", &Value(PropertyValue::Float(new_size))) => {
                self.size.clone().set(new_size as f32);
            },

            _ => ()
        }
    }
}
