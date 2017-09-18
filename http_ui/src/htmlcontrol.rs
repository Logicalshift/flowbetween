//!
//! Routines for converting UI controls to HTML
//!

use ui::*;

///
/// Trait implemented by things that can be represented by HTML
///
pub trait ToHtml {
    fn to_html(&self) -> String;
}

///
/// Returns the class for a control
///
fn control_class(ctrl: &Control) -> &str {
    use ui::ControlType::*;

    match ctrl.control_type() {
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
    fn to_html(&self) -> String {
        use ui::ControlAttribute::*;
        let mut result = String::new();

        // Start with the main div and its class
        result.push_str(&format!("<div class=\"{}\">", control_class(self)));

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            match attribute {
                &SubComponents(ref subcomponents) => {
                    // Convert any subcomponents to HTML
                    let subcomponent_html = subcomponents.iter()
                        .map(|control| control.to_html())
                        .fold(String::new(), |a, b| a + &b);
                    result.push_str(&subcomponent_html);
                },
                &Text(ref text) => {
                    result.push_str(&quote_text(text))
                },
                _ => ()
            }
        }
        
        // Close out the div and finish up
        result.push_str("</div>");

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_button_to_html() {
        assert!(Control::button().to_html() == "<div class=\"flo-button\"></div>")
    }

    #[test]
    fn can_convert_label_to_html() {
        assert!(Control::label().with("Hello & goodbye").to_html() == "<div class=\"flo-label\">Hello &amp; goodbye</div>")
    }

    #[test]
    fn can_convert_container_to_html() {
        assert!(Control::container().with(vec![Control::button()]).to_html() == "<div class=\"flo-container\"><div class=\"flo-button\"></div></div>")
    }
}
