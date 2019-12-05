use flo_binding::*;
use flo_animation::*;

///
/// Viewmodel for a layer
///
#[derive(Clone)]
pub struct LayerModel {
    /// The ID for this layer (not a binding as it never changes)
    pub id: u64,

    /// The name of this layer
    pub name: Binding<String>
}

impl PartialEq for LayerModel {
    fn eq(&self, other: &LayerModel) -> bool {
        other.id == self.id
    }
}

impl LayerModel {
    pub fn new<'a>(layer: &'a dyn Layer) -> LayerModel {
        LayerModel {
            id:     layer.id(),
            name:   bind(layer.name().unwrap_or_else(|| format!("Layer {}", layer.id())))
        }
    }
}
