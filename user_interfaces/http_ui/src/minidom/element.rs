use super::*;
use super::attribute::*;

use itertools::*;

pub struct DomElement {
    /// The name of this element
    name: String,

    /// The content of this element
    content: Vec<DomNode>
}

impl DomElement {
    ///
    /// Creates a new DOM element with the specified name
    ///
    pub fn new(name: &str) -> DomNode {
        DomNode::new(DomElement {
            name:       String::from(name),
            content:    vec![]
        })
    }
}

impl DomNodeData for DomElement {
    fn append_fragment(&self, target: &mut String) {
        // Start with '<Element'
        target.push('<');
        target.push_str(&self.name);

        // Merge any attributes that have the same name
        let attributes = self.content.iter().filter(|item| item.node_type() == DomNodeType::Attribute);
        let attributes = attributes.sorted_by_key(|attr| attr.element_name().unwrap()).into_iter();
        let attributes = attributes.group_by(|attr| attr.element_name().unwrap());
        let attributes = attributes.into_iter().map(|(name, attributes)| {
            let mut value = String::new();

            for attr in attributes {
                if value.len() > 0 { value.push_str(" "); }
                value.push_str(&attr.value().unwrap());
            }

            DomAttribute::new(&name, &value)
        });

        // Add any attributes next
        for attr in attributes {
            target.push(' ');
            attr.append_fragment(target);
        }

        // Finally, close the element
        target.push('>');

        // Append any child nodes
        for child_node in self.content.iter().filter(|item| item.node_type() != DomNodeType::Attribute) {
            child_node.append_fragment(target);
        }

        // Close the element
        target.push_str("</");
        target.push_str(&self.name);
        target.push('>');
    }

    fn node_type(&self) -> DomNodeType {
        DomNodeType::Element
    }

    fn element_name(&self) -> Option<String> {
        Some(self.name.clone())
    }

    fn content(&self) -> Vec<DomNode> {
        self.content.clone()
    }

    fn attributes(&self) -> Vec<(String, String)> {
        // Retrieve all of the child nodes that are attributes
        let attributes = self.content.iter()
            .filter(|item| item.node_type() == DomNodeType::Attribute);

        // Result is a list of (name, value) pairs
        let empty = String::from("");
        attributes
            .map(move |attr| (attr.element_name().unwrap_or(empty.clone()), attr.value().unwrap_or(empty.clone())))
            .collect()
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        // Value of the first child node that's an attribute with the specified name
        let name                = String::from(name);

        let attributes          = self.content.iter().filter(|item| item.node_type() == DomNodeType::Attribute);
        let mut matching_name   = attributes.filter(move |item| item.element_name().as_ref() == Some(&name));

        matching_name.nth(0).map(|attr| attr.value().unwrap_or(String::from("")))
    }

    fn set_attribute(&mut self, name: &str, value: &str) {
        let name                = String::from(name);
        let mut replace_index   = None;

        // Try to replace an existing attribute if we can
        for index in 0..self.content.len() {
            let node = &self.content[index];

            if node.node_type() == DomNodeType::Attribute && node.element_name().as_ref() == Some(&name) {
                // Replace this node
                replace_index = Some(index);
                break;
            }
        }

        if let Some(replace_index) = replace_index {
            // Replace an existing node if possible
            self.content[replace_index] = DomAttribute::new(&name, value);
        } else {
            // Add new attributes at the start
            self.content.insert(0, DomAttribute::new(&name, value));
        }
    }

    fn insert_child_node(&mut self, new_node: DomNode, before: usize) {
        self.content.insert(before, new_node)
    }

    fn remove_child_node(&mut self, index: usize) {
        self.content.remove(index);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_get_name() {
        let element = DomElement::new("test");

        assert!(element.element_name() == Some("test".to_string()));
        assert!(element.node_type() == DomNodeType::Element);
        assert!(element.value() == None);
    }

    #[test]
    pub fn can_append_items() {
        let mut element = DomElement::new("test");

        element.append_child_node(DomElement::new("test2"));
        element.append_child_node(DomElement::new("test3"));

        assert!(element.content().len() == 2);
    }

    #[test]
    pub fn can_get_attribute() {
        let mut element = DomElement::new("test");

        element.set_attribute("foo", "bar");

        assert!(element.content().len() > 0);
        assert!(element.get_attribute("foo").is_some());
        assert!(element.get_attribute("foo") == Some(String::from("bar")));
    }

    #[test]
    pub fn can_write_element() {
        let mut element = DomElement::new("test");

        element.set_attribute("foo", "bar");
        element.append_child_node(DomElement::new("test2"));

        let mut text = String::new();
        element.append_fragment(&mut text);

        assert!(text == "<test foo=\"bar\"><test2></test2></test>");
    }

    #[test]
    pub fn merge_elements() {
        let mut element = DomElement::new("test");

        element.append_child_node(DomAttribute::new("foo", "bar"));
        element.append_child_node(DomAttribute::new("foo", "baz"));
        element.append_child_node(DomElement::new("test2"));

        let mut text = String::new();
        element.append_fragment(&mut text);

        assert!(text == "<test foo=\"bar baz\"><test2></test2></test>");
    }
}
