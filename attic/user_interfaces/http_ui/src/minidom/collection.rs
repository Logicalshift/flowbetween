use super::*;

pub struct DomCollection(Vec<DomNode>);

impl DomCollection {
    ///
    /// Creates a new collection node
    ///
    pub fn new(items: Vec<DomNode>) -> DomNode {
        DomNode::new(DomCollection(items))
    }
}

impl DomNodeData for DomCollection {
    fn append_fragment(&self, target: &mut String) {
        for node in self.0.iter() {
            node.append_fragment(target);
        }
    }

    fn node_type(&self) -> DomNodeType {
        DomNodeType::Collection
    }

    fn content(&self) -> Vec<DomNode> {
        self.0.clone()
    }

    fn insert_child_node(&mut self, new_node: DomNode, before: usize) {
        self.0.insert(before, new_node)
    }

    fn remove_child_node(&mut self, index: usize) {
        self.0.remove(index);
    }
}
