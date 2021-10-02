use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;
use flo_canvas_animation::description::*;

use std::sync::*;
use std::time::{Duration};

///
/// Represents an animation region element
///
#[derive(Clone, Debug)]
pub struct AnimationElement(pub ElementId, pub RegionDescription);

impl VectorElement for AnimationElement {
    fn id(&self) -> ElementId { self.0 }
    fn set_id(&mut self, new_id: ElementId) { self.0 = new_id; }
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>> { None }
    fn render(&self, gc: &mut dyn GraphicsContext, properties: &VectorProperties, when: Duration) { }
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { properties }
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint> { vec![] }

    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector {
        unimplemented!()
    }
}
