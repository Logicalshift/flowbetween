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

use super::property::*;

/// Property used to describe the name of an item on the canvas
pub static PROP_NAME: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::name");

///
/// Canvas property representing the name of something
///
pub struct Name(pub String);

///
/// Converts a property value that should be a string into a string (or returns None if the property is not a valid string)
///
pub fn string_from_property(property: &CanvasProperty) -> Option<String> {
    match property {
        CanvasProperty::String(name) => Some(name.clone()),
        _                            => None
    }
}

impl From<&str> for Name {
    #[inline]
    fn from(value: &str) -> Self {
        Name(value.into())
    }
}

impl From<String> for Name {
    #[inline]
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl ToCanvasProperties for Name {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![(*PROP_NAME, CanvasProperty::String(self.0.clone()))]
    }
}

impl FromCanvasProperties for Name {
    fn used_properties() -> Vec<CanvasPropertyId> {
        vec![*PROP_NAME]
    }

    fn from_properties<'a>(properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut name_property = None;

        for (id, value) in properties {
            if id == &*PROP_NAME { name_property = Some(value); }
        }

        Some(Name(string_from_property(name_property?)?))
    }
}
