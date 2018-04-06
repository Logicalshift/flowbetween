use flo_ui::*;

use glib::object::Downcast;
use glib::translate::*;
use gdk;
use gdk::prelude::*;
use gdk_sys;

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

    /// Painting started
    PaintStart(GtkPainting),

    /// Painting continued
    PaintContinue(GtkPainting),

    /// Painting finished
    PaintFinish(GtkPainting)
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
            GtkEventParameter::None                 => ActionParameter::None,
            GtkEventParameter::ScaleValue(value)    => ActionParameter::Value(PropertyValue::Float(value)),
            GtkEventParameter::PaintStart(paint)    => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Start) ]),
            GtkEventParameter::PaintContinue(paint) => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Continue) ]),
            GtkEventParameter::PaintFinish(paint)   => ActionParameter::Paint(paint.get_device(), vec![ paint.to_painting(PaintAction::Finish) ]),
        }
    }
}

impl GtkPainting {
    ///
    /// Updates this structure from the axes in an event
    /// 
    unsafe fn update_from_axes(&mut self, axes: *const f64, device: *mut gdk_sys::GdkDevice) {
        // Turn device into a rust object
        let device: gdk::Device = gdk::Device::from_glib_borrow(device).downcast_unchecked();

        // Fetch the number of axes in the device
        let num_axes = device.get_n_axes() as usize;

        // Turn our axes pointer into a slice
        let axes = slice::from_raw_parts(axes, num_axes);

        // Get the device axes
        for axis_id in 0..num_axes {
            use gdk::AxisUse;
            let axis_value = axes[axis_id];

            // For the axes corresponding to a value in our structure, set that value
            match device.get_axis_use(axis_id as u32) {
                AxisUse::X          => { self.position.0    = axis_value; }
                AxisUse::Y          => { self.position.1    = axis_value; }
                AxisUse::Pressure   => { self.pressure      = axis_value; }
                AxisUse::Xtilt      => { self.xtilt         = axis_value; },
                AxisUse::Ytilt      => { self.ytilt         = axis_value; },

                _ => {}
            }
        }
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
