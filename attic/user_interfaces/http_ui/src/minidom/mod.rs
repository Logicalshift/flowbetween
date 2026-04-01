use std::sync::*;

mod element;
mod attribute;
mod text;
mod collection;
mod empty;
mod quote;

pub use self::element::*;
pub use self::attribute::*;
pub use self::text::*;
pub use self::collection::*;
pub use self::empty::*;

///
/// Generic DOM node container
///
#[derive(Clone)]
pub struct DomNode(Arc<RwLock<dyn DomNodeData>>);

///
/// Possible types of DOM node
///
#[derive(Clone, Copy, PartialEq)]
pub enum DomNodeType {
    /// Empty placeholder element
    Empty,

    /// <foo></foo> element
    Element,

    /// Text node
    Text,

    /// a="b" attribute
    Attribute,

    /// Simple collection of elements/text
    Collection
}

///
/// Trait implemented by objects that represent a DOM node
///
pub trait DomNodeData {
    ///
    /// Appends the text representation of this node to a string
    ///
    fn append_fragment(&self, target: &mut String);

    ///
    /// The type of this node
    ///
    fn node_type(&self) -> DomNodeType;

    ///
    /// The name of the element represented by this node
    ///
    fn element_name(&self) -> Option<String> { None }

    ///
    /// The text value of this node, if it has one
    ///
    fn value(&self) -> Option<String> { None }

    ///
    /// The content of this node
    ///
    fn content(&self) -> Vec<DomNode> { vec![] }

    ///
    /// The attributes attached to this node
    ///
    fn attributes(&self) -> Vec<(String, String)> { vec![] }

    ///
    /// Retrieves an attribute value by name
    ///
    fn get_attribute(&self, _name: &str) -> Option<String> { None }

    ///
    /// Sets an attribute in this node
    ///
    fn set_attribute(&mut self, _name: &str, _value: &str) { }

    ///
    /// Inserts a child node
    ///
    fn insert_child_node(&mut self, _new_node: DomNode, _before: usize) { }

    ///
    /// Removes a child node at a particular index
    ///
    fn remove_child_node(&mut self, _index: usize) { }

    ///
    /// Appends a child node to the end of this node
    ///
    fn append_child_node(&mut self, new_node: DomNode) {
        let len = self.content().len();
        self.insert_child_node(new_node, len);
    }

    ///
    /// Flattens this node (removing any sub-collections or empty nodes)
    ///
    fn flatten(&mut self) {
        loop {
            let content             = self.content();
            let mut replace_indexes = vec![];

            // Search the content of this node for empty or collection nodes
            for index in 0..content.len() {
                // Push this index to the appropriate collection
                match content[index].node_type() {
                    DomNodeType::Empty      => replace_indexes.push(index),
                    DomNodeType::Collection => replace_indexes.push(index),
                    _                       => ()
                };
            }

            // No more work to do if the node is completely empty
            if replace_indexes.len() == 0 {
                break;
            }

            // Replace in reverse order (keeps indexes the same)
            replace_indexes.reverse();
            for index in replace_indexes.iter() {
                // Get the item from the original content
                let original = &content[*index];

                // Remove it from this object
                self.remove_child_node(*index);

                match original.node_type() {
                    // Just remove empty nodes
                    DomNodeType::Empty      => (),

                    // Replace collection nodes with their content
                    DomNodeType::Collection => {
                        let mut collection_content = original.content();

                        while let Some(node) = collection_content.pop() {
                            self.insert_child_node(node, *index);
                        }
                    },

                    _ => panic!("Was expecting an empty or a collection type")
                }
            }
        }
    }
}

impl DomNode {
    ///
    /// Creates a new node from an item that can supply DOM data
    ///
    pub fn new<T: 'static+DomNodeData>(data: T) -> DomNode {
        DomNode(Arc::new(RwLock::new(data)))
    }

    ///
    /// Creates a new DOM node with the specified child nodes appended
    ///
    pub fn with(mut self, new_child_nodes: Vec<DomNode>) -> DomNode {
        for node in new_child_nodes {
            self.append_child_node(node)
        }
        self
    }
}

impl DomNodeData for DomNode {
    fn append_fragment(&self, target: &mut String) {
        self.0.read().unwrap().append_fragment(target)
    }

    fn node_type(&self) -> DomNodeType {
        self.0.read().unwrap().node_type()
    }

    fn element_name(&self) -> Option<String> {
        self.0.read().unwrap().element_name()
    }

    fn value(&self) -> Option<String> {
        self.0.read().unwrap().value()
    }

    fn content(&self) -> Vec<DomNode> {
        self.0.read().unwrap().content()
    }

    fn attributes(&self) -> Vec<(String, String)> {
        self.0.read().unwrap().attributes()
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        self.0.read().unwrap().get_attribute(name)
    }

    fn set_attribute(&mut self, name: &str, value: &str) {
        self.0.write().unwrap().set_attribute(name, value);
    }

    fn insert_child_node(&mut self, new_node: DomNode, before: usize) {
        self.0.write().unwrap().insert_child_node(new_node, before)
    }

    fn remove_child_node(&mut self, index: usize) {
        self.0.write().unwrap().remove_child_node(index);
    }

    fn append_child_node(&mut self, new_node: DomNode) {
        self.0.write().unwrap().append_child_node(new_node)
    }

    fn flatten(&mut self) {
        self.0.write().unwrap().flatten();
    }
}

impl ToString for DomNode {
    fn to_string(&self) -> String {
        let mut res = String::new();
        self.append_fragment(&mut res);
        res
    }
}
