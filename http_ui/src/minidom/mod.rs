use std::sync::*;

///
/// Generic DOM node container
///
#[derive(Clone)]
pub struct Node(Arc<RwLock<DomNodeData>>);

///
/// Possible types of DOM node
///
pub enum DomNodeType {
    Element,
    Text,
    Attribute
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
    fn element_name(&self) -> Option<String>;

    ///
    /// The content of this node 
    ///
    fn content(&self) -> Vec<Node>;

    ///
    /// The attributes attached to this node
    ///
    fn attributes(&self) -> Vec<(String, String)>;

    ///
    /// Retrieves an attribute value by name
    ///
    fn get_attribute(&self, name: &str) -> Option<String>;

    ///
    /// Sets an attribute in this node 
    ///
    fn set_attribute(&mut self, name: &str, value: &str);

    ///
    /// Inserts a child node
    ///
    fn insert_child_node(&mut self, new_node: Node, before: usize);

    ///
    /// Appends a child node to the end of this node
    ///
    fn append_child_node(&mut self, new_node: Node) {
        let len = self.content().len();
        self.insert_child_node(new_node, len);
    }
}

impl Node {
    ///
    /// Creates a new node from an item that can supply DOM data
    ///
    pub fn new<T: 'static+DomNodeData>(data: T) -> Node {
        Node(Arc::new(RwLock::new(data)))
    }
}

impl DomNodeData for Node {
    fn append_fragment(&self, target: &mut String) {
        self.0.read().unwrap().append_fragment(target)
    }

    fn node_type(&self) -> DomNodeType {
        self.0.read().unwrap().node_type()
    }

    fn element_name(&self) -> Option<String> {
        self.0.read().unwrap().element_name()
    }

    fn content(&self) -> Vec<Node> {
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

    fn insert_child_node(&mut self, new_node: Node, before: usize) {
        self.0.write().unwrap().insert_child_node(new_node, before)
    }

    fn append_child_node(&mut self, new_node: Node) {
        self.0.write().unwrap().append_child_node(new_node)
    }
}
