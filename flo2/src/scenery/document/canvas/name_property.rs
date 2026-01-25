use super::property::*;

/// Property used to describe the name of an item on the canvas
pub static PROP_NAME: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::name");

///
/// Canvas property representing the name of something
///
pub struct Name(pub String);

///
/// Converts a string value into a canvas property
///
pub fn string_property(string: impl Into<String>) -> CanvasProperty {
    let string = string.into();
    let bytes = string.bytes().collect::<Vec<_>>();

    CanvasProperty::ByteList(bytes)
}

///
/// Converts a property value that should be a string into a string (or returns None if the property is not a valid string)
///
pub fn string_from_property(property: &CanvasProperty) -> Option<String> {
    match property {
        CanvasProperty::ByteList(bytes) => String::from_utf8(bytes.clone()).ok(),
        _                               => None
    }
}

impl ToCanvasProperties for Name {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![(*PROP_NAME, string_property(&self.0))]
    }

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
