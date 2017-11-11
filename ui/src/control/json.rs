use super::control::*;
use super::attributes::*;
use super::super::json::*;

use serde_json::*;

impl ToJsonValue for ControlAttribute {
    fn to_json(&self) -> Value {
        use ControlAttribute::*;

        match self {
            &BoundingBox(ref bounds)            => json!({ "BoundingBox": bounds }),
            &Text(ref property)                 => json!({ "Text": property }),
            &Selected(ref property)             => json!({ "Selected": property }),
            &Id(ref id)                         => json!({ "Id": id }),
            &Controller(ref name)               => json!({ "Controller": name }),
            &Action(ref trigger, ref action)    => json!({ "Action": (trigger, action) }),

            &SubComponents(ref components)      => {
                let json_components: Vec<Value> = components.iter()
                    .map(|component| component.to_json())
                    .collect();

                json!({ "SubComponents": json_components })
            },

            &Image(ref image_resource)          => {
                // For images, we only store the ID: callers need to look it up from the resource manager in the controller that made this control
                json!({ 
                    "Image": {
                        "id":   image_resource.id(),
                        "name": image_resource.name()
                    }
                })
            }
        }
    }
}

impl ToJsonValue for Control {
    fn to_json(&self) -> Value {
        let attributes: Vec<Value> = self.attributes()
            .map(|attribute| attribute.to_json())
            .collect();

        json!({
            "attributes":   attributes,
            "control_type": self.control_type()
        })
    }
}
