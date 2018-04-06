use flo_ui::*;

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
