use super::*;

///
/// Represents a text element in the DOM
/// 
pub struct DomText(String);

impl DomText {
    ///
    /// Creates a new text node
    /// 
    pub fn new(text: &str) -> DomNode {
        DomNode::new(DomText(String::from(text)))
    }
}

impl DomNodeData for DomText {
    fn append_fragment(&self, target: &mut String) {
        unimplemented!()
    }

    fn node_type(&self) -> DomNodeType {
        DomNodeType::Text
    }

    fn value(&self) -> Option<String> { 
        Some(self.0.clone())
     }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn text_properties_are_correct() {
        let text = DomText::new("test");

        assert!(text.node_type() == DomNodeType::Text);
        assert!(text.value() == Some(String::from("test")));
    }
}