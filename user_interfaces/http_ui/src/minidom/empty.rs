use super::*;

pub struct DomEmpty;

impl DomEmpty {
    ///
    /// Creates a new collection node
    ///
    pub fn new() -> DomNode {
        DomNode::new(DomEmpty)
    }
}

impl DomNodeData for DomEmpty {
    fn append_fragment(&self, _target: &mut String) {
    }

    fn node_type(&self) -> DomNodeType {
        DomNodeType::Empty
    }
}
