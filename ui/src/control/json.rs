use super::control::*;
use super::attributes::*;
use super::super::json::*;

use serde_json::*;

impl ToJsonValue for ControlAttribute {
    fn to_json(&self) -> Value {
        use ControlAttribute::*;
        use crate::Appearance::*;
        use crate::State::*;

        match self {
            BoundingBox(bounds)                     => json!({ "BoundingBox": bounds }),
            Text(property)                          => json!({ "Text": property }),
            ZIndex(zindex)                          => json!({ "ZIndex": zindex }),
            Padding((left, top), (right, bottom))   => json!({ "Padding": { "left": left, "top": top, "right": right, "bottom": bottom } }),
            FontAttr(attr)                          => json!({ "Font": attr }),
            StateAttr(Selected(property))           => json!({ "Selected": property }),
            StateAttr(Badged(property))             => json!({ "Badged": property }),
            StateAttr(Value(property))              => json!({ "Value": property }),
            StateAttr(Range((min, max)))            => json!({ "Range": [min, max] }),
            StateAttr(Enabled(property))            => json!({ "Enabled": property }),
            StateAttr(FocusPriority(property))      => json!({ "FocusPriority": property }),
            PopupAttr(popup)                        => json!({ "Popup": popup }),
            ScrollAttr(scroll)                      => json!({ "Scroll": scroll }),
            Id(id)                                  => json!({ "Id": id }),
            Controller(name)                        => json!({ "Controller": name }),
            Action(trigger, action)                 => json!({ "Action": (trigger, action) }),
            HintAttr(hint)                          => json!({ "Hint": hint }),

            SubComponents(components)               => {
                let json_components: Vec<_> = components.iter()
                    .map(|component| component.to_json())
                    .collect();

                json!({ "SubComponents": json_components })
            },

            AppearanceAttr(Image(image_resource))   => {
                // For images, we only store the ID: callers need to look it up from the resource manager in the controller that made this control
                json!({
                    "Image": {
                        "id":   image_resource.id(),
                        "name": image_resource.name()
                    }
                })
            },

            AppearanceAttr(Background(color))       => json!({ "Background": color.to_rgba_components() }),
            AppearanceAttr(Foreground(color))       => json!({ "Foreground": color.to_rgba_components() }),

            Canvas(canvas_resource)                 => {
                json!({
                    "Image": {
                        "id":   canvas_resource.id(),
                        "name": canvas_resource.name()
                    }
                })
            }
        }
    }
}

impl ToJsonValue for Control {
    fn to_json(&self) -> Value {
        let attributes: Vec<_> = self.attributes()
            .map(|attribute| attribute.to_json())
            .collect();

        json!({
            "attributes":   attributes,
            "control_type": self.control_type()
        })
    }
}
