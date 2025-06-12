use super::control_id::*;

use ::serde::*;

use std::ops::{Range};

///
/// Specifies the type of a control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ControlType {
    Label,
    Button,
    Checkbox,
    ProgressBar,
    Spinner,
    RadioButton,
    Separator,
    Slider(Range<f64>),
}

///
/// Specifies the value set for a control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum ControlValue {
    Checked(bool),
    Text(String),
    Integer(String),
    Float(f64),
}

///
/// Event sent to the owner of a dialog when a control action occurs
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum ControlEvent {
    Pressed(ControlId),
    SetValue(ControlId, ControlValue),
}
