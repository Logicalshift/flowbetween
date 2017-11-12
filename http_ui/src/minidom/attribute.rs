use super::*;

pub struct DomAttribute(String, String);

impl DomAttribute {
    pub fn new(name: &str, value: &str) -> DomNode {
        DomNode::new(DomAttribute(String::from(name), String::from(value)))
    }
}

impl DomNodeData for DomAttribute {
    fn append_fragment(&self, target: &mut String) {
        unimplemented!()
    }

    fn node_type(&self) -> DomNodeType {
        DomNodeType::Attribute
    }

    fn element_name(&self) -> Option<String> {
        Some(self.0.clone())
    }

    fn value(&self) -> Option<String> {
        Some(self.1.clone())
    }
}

#[cfg(Test)]
mod test {
    use super::*;

    #[test]
    fn type_is_right() {
        assert!(DomAttribute::new("foo", "bar").node_type() == ElementType::Attribute)
    }

    #[test]
    fn can_read_name() {
        assert!(DomAttribute::new("foo", "bar").element_name() == Some("foo".to_string()))
    }

    #[test]
    fn can_read_value() {
        assert!(DomAttribute::new("foo", "bar").value() == Some("bar".to_string()))
    }
}
