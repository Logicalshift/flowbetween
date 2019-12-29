use super::super::*;

use flo_binding::*;
use std::sync::*;

///
/// Controller that provides standard behaviour for popups.
/// Supply this as a controller for a control that needs a popup.
/// This will supply the popup container and suppress the popup
/// UI while the popup is not visible.
///
pub struct PopupController<ContentController: Controller> {
    /// Controller that provides the main content for the popup
    content_controller: ContentController,

    /// Binding that provides the size of the popup
    popup_size: BindRef<(u32, u32)>,

    /// The direction the popup will appear in
    popup_direction: BindRef<PopupDirection>,

    /// The offset of the popup from the parent control
    offset: BindRef<u32>,

    /// Binding that specifies whether or not the popup is open
    /// (This will set to false if the popup is dismissed)
    open: Binding<bool>,

    /// User interface for this controller
    ui: BindRef<Control>
}

impl<ContentController: Controller> PopupController<ContentController> {
    ///
    /// Creates a new popup controller.
    ///
    /// Default settings are a size of 100,100, popup center, offset 8
    ///
    pub fn new(controller: ContentController, is_open: &Binding<bool>) -> PopupController<ContentController> {
        // Create the initial set of bindings
        let content_controller  = controller;
        let open                = is_open.clone();
        let popup_size          = BindRef::from(&(100, 100));
        let popup_direction     = BindRef::from(&PopupDirection::WindowCentered);
        let offset              = BindRef::from(&8);
        let content             = content_controller.ui();

        // Derive the basic UI binding
        let ui                  = Self::create_ui(&content, &open, &popup_size, &popup_direction, &offset);

        // Generate the popup controller
        PopupController {
            content_controller: content_controller,
            open:               open,
            popup_size:         popup_size,
            popup_direction:    popup_direction,
            offset:             offset,
            ui:                 ui
        }
    }

    ///
    /// Returns a modified controller with a different size
    ///
    pub fn with_size<T: Into<BindRef<(u32, u32)>>>(mut self, size: T) -> PopupController<ContentController> {
        self.popup_size = size.into();
        self.regenerate_ui()
    }

    ///
    /// Returns a modified controller with a different direction
    ///
    pub fn with_direction<T: Into<BindRef<PopupDirection>>>(mut self, direction: T) -> PopupController<ContentController> {
        self.popup_direction = direction.into();
        self.regenerate_ui()
    }

    ///
    /// Returns a modified controller with a different offset
    ///
    pub fn with_offset<T: Into<BindRef<u32>>>(mut self, offset: T) -> PopupController<ContentController> {
        self.offset = offset.into();
        self.regenerate_ui()
    }

    ///
    /// Regenerates the UI field from the current bindings
    ///
    fn regenerate_ui(mut self) -> PopupController<ContentController> {
        self.ui = Self::create_ui(&self.content_controller.ui(), &self.open, &self.popup_size, &self.popup_direction, &self.offset);
        self
    }

    ///
    /// Creates the UI binding for this controller
    ///
    fn create_ui(content: &BindRef<Control>, open: &Binding<bool>, size: &BindRef<(u32, u32)>, direction: &BindRef<PopupDirection>, offset: &BindRef<u32>) -> BindRef<Control> {
        // Clone the model bits
        let content     = content.clone();
        let open        = open.clone();
        let size        = size.clone();
        let direction   = direction.clone();
        let offset      = offset.clone();

        // Compute the UI
        BindRef::from(computed(move || {
            // Get the standard popup properties
            let open            = open.get();
            let direction       = direction.get();
            let (width, height) = size.get();
            let offset          = offset.get();

            if !open {
                // If not open, then generate an empty popup
                // Not binding the UI here so if the controller updates, nothing happens
                Control::popup()
                    .with(Popup::IsOpen(Property::Bool(false)))
                    .with(Popup::Direction(direction))
                    .with(Popup::Size(width, height))
                    .with(Popup::Offset(offset))
                    .with(ControlAttribute::Padding((8,8), (8,8)))
                    .with(ControlAttribute::ZIndex(1000))
            } else {
                // Bind the UI only when the controller is open
                Control::popup()
                    .with(Popup::IsOpen(Property::Bool(true)))
                    .with(Popup::Direction(direction))
                    .with(Popup::Size(width, height))
                    .with(Popup::Offset(offset))
                    .with(ControlAttribute::Padding((8,8), (8,8)))
                    .with(ControlAttribute::ZIndex(1000))
                    .with((ActionTrigger::Dismiss, "DismissPopup"))
                    .with(vec![
                        content.get()
                    ])
            }
        }))
    }
}

impl<ContentController: Controller> Controller for PopupController<ContentController> {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        self.content_controller.get_viewmodel()
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        self.content_controller.get_subcontroller(id)
    }

    fn action(&self, action_id: &str, action_data: &ActionParameter) {
        // Hide the popup if our DismissPopup action is fired
        if action_id == "DismissPopup" {
            self.open.set(false);
        }

        // Pass the action on to the main controller
        self.content_controller.action(action_id, action_data);
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        self.content_controller.get_image_resources()
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        self.content_controller.get_canvas_resources()
    }
}
