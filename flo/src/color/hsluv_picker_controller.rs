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

    viewmodel:  Arc<DynamicViewModel>
}

impl HsluvPickerController {
    ///
    /// Creates a new HSLUV colour picker controller
    /// 
    pub fn new(color: &Binding<Color>) -> HsluvPickerController {
        let images      = ResourceManager::new();
        let color       = color.clone();
        let viewmodel   = DynamicViewModel::new();

        // Set up the images
        let hsluv_wheel = HSLUV_COLOR_WHEEL.clone();
        let hsluv_wheel = images.register(hsluv_wheel);
        images.assign_name(&hsluv_wheel, "Wheel");

        // Set up the UI
        let ui          = Self::create_ui(&color, &hsluv_wheel);
        
        // Controller is ready to go
        HsluvPickerController {
            ui:         ui,
            images:     Arc::new(images),
            viewmodel:  Arc::new(viewmodel)
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn create_ui(color: &Binding<Color>, hsluv_wheel: &Resource<Image>) -> BindRef<Control> {
        let color = color.clone();

        BindRef::from(&Control::empty()
            .with(Bounds::fill_all())
            .with(hsluv_wheel.clone()))
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
        None
    }
}
