use super::control_id::*;

use flo_binding::*;
use ::serde::*;
use serde::de::{Error as DeError};
use serde::ser::{Error as SeError};

use std::ops::{Range};

///
/// Specifies the type of a control
///
#[derive(Clone)]
pub enum ControlType {
    Label(BindRef<String>),
    Button(BindRef<String>),
    Checkbox(BindRef<String>),
    ProgressBar,
    Spinner,
    RadioButton(BindRef<String>),
    Separator,
    Slider(BindRef<Range<f64>>),
}

///
/// Specifies the value set for a control
///
#[derive(Clone)]
pub enum ControlValue {
    None,
    Checked(Binding<bool>),
    Text(Binding<String>),
    Integer(Binding<i64>),
    Float(Binding<f64>),
}

///
/// Event sent to the owner of a dialog when a control action occurs
///
#[derive(Serialize, Deserialize, Clone)]
pub enum ControlEvent {
    Pressed(ControlId),
    SetValue(ControlId, ControlValue),
}

impl Serialize for ControlType {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        Err(S::Error::custom("ControlType cannot be serialized"))
    }
}

impl<'a> Deserialize<'a> for ControlType {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a> 
    {
        Err(D::Error::custom("ControlType cannot be serialized"))
    }
}

impl Serialize for ControlValue {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        Err(S::Error::custom("ControlValue cannot be serialized"))
    }
}

impl<'a> Deserialize<'a> for ControlValue {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a> 
    {
        Err(D::Error::custom("ControlValue cannot be serialized"))
    }
}
