//!
//! Routines for converting UI controls to HTML
//!

use ui::*;

///
/// Trait implemented by things that can be represented by HTML
///
pub trait ToHtml {
    fn to_html(&self, base_path: &str) -> String;
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

///
/// Quotes text for HTML display
///
fn quote_text(text: &String) -> String {
    let mut result = String::new();

    for c in text.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '\'' => result.push_str("&quot;"),

            _ => result.push(c)
        }
    }

    result
}

impl ToHtml for Control {
    fn to_html(&self, base_path: &str) -> String {
        use ui::ControlAttribute::*;
        let mut result = String::new();

        // Start with the main element
        result.push_str(&format!("<{}>", control_class(self)));

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            match attribute {
                &SubComponents(ref subcomponents) => {
                    // Subcomponents go inside the div
                    let subcomponent_html = subcomponents.iter()
                        .map(|control| control.to_html(base_path))
                        .fold(String::new(), |a, b| a + &b);
                    result.push_str(&subcomponent_html);
                },

                &Text(ref text) => {
                    // Any text is just the text attached to the div
                    result.push_str(&quote_text(&text.to_string()))
                },

                _ => ()
            }
        }
        
        // Close out the element and finish up
        result.push_str(&format!("</{}>", control_class(self)));

        result
    }
}

impl ToHtml for () {
    fn to_html(&self, base_url: &str) -> String {
        String::from("")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_button_to_html() {
        assert!(Control::button().to_html("") == "<flo-button></flo-button>")
    }

    #[test]
    fn can_convert_label_to_html() {
        assert!(Control::label().with("Hello & goodbye").to_html("") == "<flo-label>Hello &amp; goodbye</flo-label>")
    }

    #[test]
    fn can_convert_container_to_html() {
        assert!(Control::container().with(vec![Control::button()]).to_html("") == "<flo-container><flo-button></flo-button></flo-container>")
    }
}
