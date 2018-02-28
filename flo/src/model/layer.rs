use binding::*;
use animation::*;

///
/// Viewmodel for a layer
/// 
#[derive(Clone)]
pub struct LayerModel {
    /// The ID for this layer
    pub id: Binding<u64>,

    /// The name of this layer
    pub name: Binding<String>
}

impl PartialEq for LayerModel {
    fn eq(&self, other: &LayerModel) -> bool {
        other.id.get() == self.id.get()
    }
}

impl LayerModel {
    pub fn new<'a>(layer: &Reader<'a, Layer>) -> LayerModel {
        LayerModel {
            id:     bind(layer.id()),
            name:   bind(format!("Layer {}", layer.id()))
        }
    }
}