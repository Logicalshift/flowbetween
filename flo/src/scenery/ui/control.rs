// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
