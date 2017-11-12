//!
//! Routines for converting UI controls to HTML
//!

use ui::*;
use super::minidom::*;

use percent_encoding::*;

///
/// Trait implemented by things that can be represented by HTML
///
pub trait ToHtml {
    fn to_html(&self, base_path: &str) -> DomNode;
}

///
/// Returns the class for a control
///
fn control_class(ctrl: &Control) -> &str {
    use ui::ControlType::*;

    match ctrl.control_type() {
        Empty       => "flo-empty",
        Container   => "flo-container",
        Button      => "flo-button",
        Label       => "flo-label"
    }
}

impl ToHtml for Control {
    fn to_html(&self, base_path: &str) -> DomNode {
        // Start with the main element
        let mut result = DomElement::new(control_class(self));

        // The base path changes when the controller changes
        let new_path;
        let mut subcomponent_path   = base_path;

        if let Some(subcontroller_name) = self.controller() {
            new_path            = format!("{}/{}", base_path, utf8_percent_encode(subcontroller_name, DEFAULT_ENCODE_SET));
            subcomponent_path   = &new_path;
        }

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            result.append_child_node(attribute.to_html(subcomponent_path));
        }

        result
    }
}

impl ToHtml for ControlAttribute {
    fn to_html(&self, base_path: &str) -> DomNode {
        use ui::ControlAttribute::*;

        match self {
            &SubComponents(ref subcomponents) => {
                let mut result = DomCollection::new(vec![]);

                // Subcomponents go inside the div
                let subcomponent_nodes = subcomponents.iter()
                    .map(|control| control.to_html(base_path));
                
                for node in subcomponent_nodes { 
                    result.append_child_node(node);
                }

                result
            },

            &Text(ref text) => DomText::new(&text.to_string()),

            &Image(ref image) => {
                // Use the image's name if it has one, otherwise the ID
                let image_name = {
                    if let Some(name) = image.name() {
                        name
                    } else {
                        image.id().to_string()
                    }
                };

                // Build the URL from the base path
                let image_url = format!("{}/{}", base_path, utf8_percent_encode(&image_name, DEFAULT_ENCODE_SET));

                // Style attribute to render this image as the background
                DomAttribute::new("style", &format!("background: no-repeat center/100% url({});", image_url))
            }

            _ => DomEmpty::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_button_to_html() {
        assert!(Control::button().to_html("").to_string() == "<flo-button></flo-button>")
    }

    #[test]
    fn can_convert_label_to_html() {
        assert!(Control::label().with("Hello & goodbye").to_html("").to_string() == "<flo-label>Hello &amp; goodbye</flo-label>")
    }

    #[test]
    fn can_convert_container_to_html() {
        assert!(Control::container().with(vec![Control::button()]).to_html("").to_string() == "<flo-container><flo-button></flo-button></flo-container>")
    }
}
