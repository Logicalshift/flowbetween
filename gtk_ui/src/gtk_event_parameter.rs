use flo_ui::*;

///
/// Parameters that are available for a GTK event
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum GtkEventParameter {
    /// Event has no extra data
    None,

    /// Event indicates the value set for a scale
    ScaleValue(f64)
}

impl From<GtkEventParameter> for ActionParameter {
    fn from(event: GtkEventParameter) -> ActionParameter {
        match event {
            GtkEventParameter::None                 => ActionParameter::None,
            GtkEventParameter::ScaleValue(value)    => ActionParameter::Value(PropertyValue::Float(value))
        }
    }
}
