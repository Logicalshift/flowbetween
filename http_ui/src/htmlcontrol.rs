//!
//! Routines for converting UI controls to HTML
//!

use ui::*;
use super::minidom::*;

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
        use ui::ControlAttribute::*;

        // Start with the main element
        let mut result = DomElement::new(control_class(self));

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            match attribute {
                &SubComponents(ref subcomponents) => {
                    // Subcomponents go inside the div
                    let subcomponent_nodes = subcomponents.iter()
                        .map(|control| control.to_html(base_path));
                    
                    for node in subcomponent_nodes { 
                        result.append_child_node(node);
                    }
                },

                &Text(ref text) => {
                    // Any text is just the text attached to the div
                    result.append_child_node(DomText::new(&text.to_string()));
                },

                _ => ()
            }
        }

        result
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
