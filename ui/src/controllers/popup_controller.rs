use super::super::*;

use binding::*;

///
/// Controller that provides standard behaviour for popups.
/// Supply this as a controller for a control that needs a popup.
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
    open: Binding<bool>
}

impl<ContentController: Controller> PopupController<ContentController> {
    ///
    /// Creates a new popup controller.
    /// 
    /// Default settings are a size of 100,100, popup center, offset 8
    /// 
    pub fn new(controller: ContentController, is_open: &Binding<bool>) -> PopupController<ContentController> {
        PopupController {
            content_controller: controller,
            open:               is_open.clone(),
            popup_size:         BindRef::from(bind((100, 100))),
            popup_direction:    BindRef::from(bind(PopupDirection::WindowCentered)),
            offset:             BindRef::from(bind(8))
        }
    }

    ///
    /// Returns a modified controller with a different size
    /// 
    pub fn with_size<T: Into<BindRef<(u32, u32)>>>(mut self, size: T) -> PopupController<ContentController> {
        self.popup_size = size.into();
        self
    }

    ///
    /// Returns a modified controller with a different direction
    /// 
    pub fn with_direction<T: Into<BindRef<PopupDirection>>>(mut self, direction: T) -> PopupController<ContentController> {
        self.popup_direction = direction.into();
        self
    }

    ///
    /// Returns a modified controller with a different offset
    /// 
    pub fn with_offset<T: Into<BindRef<u32>>>(mut self, offset: T) -> PopupController<ContentController> {
        self.offset = offset.into();
        self
    }
}
