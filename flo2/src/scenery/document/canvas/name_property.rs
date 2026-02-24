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
