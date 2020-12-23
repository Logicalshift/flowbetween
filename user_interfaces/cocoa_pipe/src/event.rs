use super::action::*;

use flo_ui::*;

///
/// Data provided by a point during a painting action
///
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)] pub struct AppPainting {
    pub pointer_id: i32,
    pub position_x: f64,
    pub position_y: f64,
    pub pressure:   f64,
    pub tilt_x:     f64,
    pub tilt_y:     f64
}

unsafe impl objc::Encode for AppPainting {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("{AppPainting=iddddd}") }
    }
}

///
/// Action for editing a view
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EditAction {
    /// View is being live edited
    LiveEditing = 0,

    /// View value is being set
    ValueSet = 1
}

///
/// Enumeration of events that can be generated by a Cocoa UI
///
#[derive(Clone, PartialEq, Debug)]
pub enum AppEvent {
    /// The cocoa UI has finished updates and is about to return control to the user (triggered by a RequestTick action)
    Tick,

    /// Request for the UI to stop sending updates (eg, because we're already refreshing canvases)
    SuspendUpdates,

    /// Notification that we've finished updating and the UI can send more information
    ResumeUpdates,

    /// User has clicked on a view
    Click(usize, String),

    /// User has dismissed a view
    Dismiss(usize, String),

    /// User has focused a view
    Focus(usize, String),

    /// User is editing a view
    EditValue(usize, String, EditAction, PropertyValue),

    /// The scrolling region has changed
    VirtualScroll(usize, String, (u32, u32), (u32, u32)),

    /// Indicates that a point has been dragged to another location
    Drag(usize, String, DragAction, (f64, f64), (f64, f64)),

    /// A painting action has started with the device in the specified state
    PaintStart(usize, String, AppPaintDevice, AppPainting),

    /// A painting action is continuing
    PaintContinue(usize, String, AppPaintDevice, AppPainting),

    /// The painting action finished successfully
    PaintFinish(usize, String, AppPaintDevice, AppPainting),

    /// The painting action was cancelled
    PaintCancel(usize, String, AppPaintDevice, AppPainting)
}

impl AppPainting {
    ///
    /// Converts a cocoa painting event into a UI painting event
    ///
    pub fn into_painting(&self, action: PaintAction) -> Painting {
        Painting {
            action:         action,
            pointer_id:     self.pointer_id,
            modifier_keys:  vec![],
            location:       (self.position_x as f32, self.position_y as f32),
            pressure:       self.pressure as f32,
            tilt_x:         self.tilt_x as f32,
            tilt_y:         self.tilt_y as f32
        }
    }
}
