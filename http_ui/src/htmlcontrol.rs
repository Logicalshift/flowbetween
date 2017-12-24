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
    fn to_html(&self, base_path: &str) -> DomNode {
        self.to_html_subcomponent(base_path, "")
    }

    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode;
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
        Label       => "flo-label",
        Canvas      => "flo-canvas"
    }
}

impl ToHtml for Control {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        // Start with the main element
        let mut result = DomElement::new(control_class(self));

        // The base path changes when the controller changes
        let new_path;
        let mut subcomponent_path   = controller_path;

        if let Some(subcontroller_name) = self.controller() {
            new_path                = format!("{}/{}", controller_path, utf8_percent_encode(subcontroller_name, DEFAULT_ENCODE_SET));
            subcomponent_path       = &new_path;
        }

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            result.append_child_node(attribute.to_html_subcomponent(base_path, subcomponent_path));
        }

        // Flatten to create a 'clean' DOM without collections or empty nodes
        result.flatten();
        result
    }
}

impl ToHtml for ControlAttribute {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        use ui::ControlAttribute::*;

        match self {
            &SubComponents(ref subcomponents) => {
                let mut result = DomCollection::new(vec![]);

                // Subcomponents go inside the div
                let subcomponent_nodes = subcomponents.iter()
                    .map(|control| control.to_html_subcomponent(base_path, controller_path));
                
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
                let image_url = format!("{}/i{}/{}", base_path, controller_path, utf8_percent_encode(&image_name, DEFAULT_ENCODE_SET));

                // Style attribute to render this image as the background
                DomAttribute::new("style", &format!("background: no-repeat center/contain url('{}');", image_url))
            },


            &Canvas(ref canvas) => {
                // Use the canvas's name if it has one, otherwise the ID
                let canvas_name = {
                    if let Some(name) = canvas.name() {
                        name
                    } else {
                        canvas.id().to_string()
                    }
                };

                // Build the URL from the base path
                let canvas_url = format!("{}/c{}/{}", base_path, controller_path, utf8_percent_encode(&canvas_name, DEFAULT_ENCODE_SET));

                // We attach canvas details to the node when it has a canvas attached to it
                DomCollection::new(vec![
                    DomAttribute::new("flo-canvas",     &canvas_url),
                    DomAttribute::new("flo-name",       &canvas_name),
                    DomAttribute::new("flo-controller", if controller_path.len() > 0 { &controller_path[1..] } else { "" })
                ])
            }

            &Background(ref col) => {
                let (r, g, b, a)    = col.to_rgba();
                let (r, g, b)       = ((r*255.0).floor() as i32, (g*255.0).floor() as i32, (b*255.0).floor() as i32);

                DomAttribute::new("style", &format!("background-color: rgba({}, {}, {}, {});", r, g, b, a))
            },

            &FontSize(size) => {
                DomAttribute::new("style", &format!("font-size: {}px;", size))
            },

            &BoundingBox(_) => DomEmpty::new(),
            &Selected(_)    => DomEmpty::new(),
            &Id(_)          => DomEmpty::new(),
            &Controller(_)  => DomEmpty::new(),
            &Action(_, _)   => DomEmpty::new()
        }
    }
}

#[cfg(test)]
mod test {
    use ui::canvas::*;
    use super::*;
    use std::sync::*;

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

    #[test]
    fn can_convert_canvas_to_html() {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(Canvas::new());
        resource_manager.assign_name(&canvas, "test_canvas");

        let control = Control::canvas().with(canvas.clone());

        assert!(control.to_html("test/base").to_string() == "<flo-canvas flo-canvas=\"test/base/c/test_canvas\" flo-name=\"test_canvas\" flo-controller=\"\"></flo-canvas>")
    }

    #[test]
    fn canvas_includes_controller() {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(Canvas::new());
        resource_manager.assign_name(&canvas, "test_canvas");

        let control = Control::empty()
            .with_controller("Test")
            .with(vec![Control::canvas().with(canvas.clone())]);

        println!("{}", control.to_html("test/base").to_string());
        assert!(control.to_html("test/base").to_string() == "<flo-empty><flo-canvas flo-canvas=\"test/base/c/Test/test_canvas\" flo-name=\"test_canvas\" flo-controller=\"Test\"></flo-canvas></flo-empty>");
    }

    #[test]
    fn image_includes_controller() {
        let resource_manager    = ResourceManager::new();
        let image               = resource_manager.register(Image::Png(Arc::new(InMemoryImageData::new(vec![]))));
        resource_manager.assign_name(&image, "test_image");

        let control = Control::empty()
            .with_controller("Test")
            .with(vec![Control::empty().with(image)]);

        println!("{}", control.to_html("test/base").to_string());
        assert!(control.to_html("test/base").to_string() == "<flo-empty><flo-empty style=\"background: no-repeat center/contain url(&quot;test/base/i/Test/test_image&quot;);\"></flo-empty></flo-empty>");
    }
}
