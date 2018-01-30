use binding::*;
use animation::*;

///
/// Viewmodel for a layer
/// 
#[derive(Clone)]
pub struct LayerViewModel {
    /// The ID for this layer
    pub id: Binding<u64>,

    /// The name of this layer
    pub name: Binding<String>
}

impl PartialEq for LayerViewModel {
    fn eq(&self, other: &LayerViewModel) -> bool {
        other.id.get() == self.id.get()
    }
}

impl LayerViewModel {
    pub fn new<'a>(layer: &Reader<'a, Layer>) -> LayerViewModel {
        LayerViewModel {
            id:     bind(layer.id()),
            name:   bind(format!("Layer {}", layer.id()))
        }
    }
}