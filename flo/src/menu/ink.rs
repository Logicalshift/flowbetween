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
    canvases:   Arc<ResourceManager<BindingCanvas>>,
    ui:         BindRef<Control>,
    view_model: Arc<NullViewModel>
}

impl InkMenuController {
    ///
    /// Creates a new ink menu controller
    /// 
    pub fn new(size: &Binding<f32>, opacity: &Binding<f32>, color: &Binding<Color>) -> InkMenuController {
        // Create the canvases
        let canvases = Arc::new(ResourceManager::new());

        let preview = Self::brush_preview(size, opacity, color);
        let preview = canvases.register(preview);
        canvases.assign_name(&preview, "Preview");

        // Generate the UI
        let ui = BindRef::from(bind(Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::canvas()
                        .with(preview)
                        .with(Bounds::next_horiz(64.0))
                ])));

        // Finalize the control
        InkMenuController {
            canvases:   canvases, 
            ui:         ui,
            view_model: Arc::new(NullViewModel::new())
        }
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
}
