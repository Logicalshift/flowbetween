use flo_ui::*;

use glib::translate::*;
use gdk;
use gdk_sys;
use cairo;

use std::slice;

///
/// Parameters that are available for a GTK event
///
#[derive(Clone, PartialEq, Debug)]
pub enum GtkEventParameter {
    /// Event has no extra data
    None,

    /// Event indicates the value set for a scale
    ScaleValue(f64),

    /// Event indicating that whether or not the control is selected has changed
    SelectedValue(bool),

    /// New text for a control
    NewText(String),

    /// Painting started
    PaintStart(GtkPainting),

    /// Painting continued
    PaintContinue(GtkPainting),

    /// Painting finished
    PaintFinish(GtkPainting),

    /// Painting cancelled
    PaintCancel(PaintDevice),

    /// User has started dragging over a widget
    DragStart(f64, f64),

    /// User is continuing to drag a widget
    DragContinue((f64, f64), (f64, f64)),

    /// User has finished dragging a widget
    DragFinish((f64, f64), (f64, f64)),

    /// Virtual scroll region has moved (tuples are the x and y coordinates and the width and height of the grid)
    VirtualScroll((u32, u32), (u32, u32))
}

///
/// Parameters for a painting event
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GtkPainting {
    pub position:   (f64, f64),
    pub pressure:   f64,
    pub xtilt:      f64,
    pub ytilt:      f64
}

impl From<GtkEventParameter> for ActionParameter {
    fn from(event: GtkEventParameter) -> ActionParameter {
        match event {
            GtkEventParameter::None                                         => ActionParameter::None,
            GtkEventParameter::ScaleValue(value)                            => ActionParameter::Value(PropertyValue::Float(value)),
            GtkEventParameter::SelectedValue(value)                         => ActionParameter::Value(PropertyValue::Bool(value)),
            GtkEventParameter::NewText(value)                               => ActionParameter::Value(PropertyValue::String(value)),
            GtkEventParameter::PaintStart(paint)                            => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Start) ]),
            GtkEventParameter::PaintContinue(paint)                         => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Continue) ]),
            GtkEventParameter::PaintFinish(paint)                           => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Finish) ]),
            GtkEventParameter::PaintCancel(device)                          => ActionParameter::Paint(device, vec![]),
            GtkEventParameter::DragStart(x, y)                              => ActionParameter::Drag(DragAction::Start, (x as f32, y as f32), (x as f32, y as f32)),
            GtkEventParameter::DragContinue((from_x, from_y), (to_x, to_y)) => ActionParameter::Drag(DragAction::Drag, (from_x as f32, from_y as f32), (to_x as f32, to_y as f32)),
            GtkEventParameter::DragFinish((from_x, from_y), (to_x, to_y))   => ActionParameter::Drag(DragAction::Finish, (from_x as f32, from_y as f32), (to_x as f32, to_y as f32)),
            GtkEventParameter::VirtualScroll(top_left, size)                => ActionParameter::VirtualScroll(top_left, size)
        }
    }
}

impl GtkPainting {
    ///
    /// Updates this structure from the axes in an event
    ///
    unsafe fn update_from_axes(&mut self, axes: *const f64, device: *mut gdk_sys::GdkDevice) {
        // Turn device into a rust object
        let device: Borrowed<gdk::Device> = gdk::Device::from_glib_borrow(device);

        // Fetch the number of axes in the device
        let num_axes = if axes.is_null() { return; } else { device.get_n_axes() as usize };

        // Turn our axes pointer into a slice
        let axes = slice::from_raw_parts(axes, num_axes);

        // Get the device axes
        for axis_id in 0..num_axes {
            use gdk::AxisUse;
            let axis_value = axes[axis_id];

            // For the axes corresponding to a value in our structure, set that value
            match device.get_axis_use(axis_id as u32) {
                /*
                AxisUse::X          => { self.position.0    = axis_value; },
                AxisUse::Y          => { self.position.1    = axis_value; },
                */
                AxisUse::Pressure   => { self.pressure      = axis_value; },
                AxisUse::Xtilt      => { self.xtilt         = axis_value; },
                AxisUse::Ytilt      => { self.ytilt         = axis_value; },

                _ => {}
            }
        }
    }

    ///
    /// Transforms this painting event using a matrix
    ///
    pub fn transform(&mut self, matrix: &cairo::Matrix) {
        self.position = matrix.transform_point(self.position.0, self.position.1);
    }

    ///
    /// Creates a painting action from a motion event
    ///
    pub fn from_button(button: &gdk::EventButton) -> GtkPainting {
        // Create a neutral painting
        let mut painting = GtkPainting {
            position: button.get_position(),
            pressure: 1.0,
            xtilt: 0.0,
            ytilt: 0.0
        };

        // Update from the axes available from this device
        unsafe {
            let button = button.as_ref();
            painting.update_from_axes(button.axes, button.device);
        }

        // Result should be populated now
        painting
    }

    ///
    /// Creates a painting action from a motion event
    ///
    pub fn from_motion(motion: &gdk::EventMotion) -> GtkPainting {
        // Create a neutral painting
        let mut painting = GtkPainting {
            position: motion.get_position(),
            pressure: 1.0,
            xtilt: 0.0,
            ytilt: 0.0
        };

        // Update from the axes available from this device
        unsafe {
            let motion = motion.as_ref();
            painting.update_from_axes(motion.axes, motion.device);
        }

        // Result should be populated now
        painting
    }

    ///
    /// Turns this into an indicator of the device that performed the painting action
    ///
    pub fn get_device(&self) -> PaintDevice {
        // TODO: actually determine the device that's in use
        PaintDevice::Mouse(MouseButton::Left)
    }

    ///
    /// Turns this into a UI Painting object with a particular action
    ///
    pub fn to_painting(&self, action: PaintAction) -> Painting {
        let (x, y) = self.position;

        Painting {
            action:     action,
            pointer_id: 0,
            location:   (x as f32, y as f32),
            pressure:   self.pressure as f32,
            tilt_x:     self.xtilt as f32,
            tilt_y:     self.ytilt as f32
        }
    }
}
