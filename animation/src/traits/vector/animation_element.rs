use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;
use flo_canvas_animation::*;
use flo_canvas_animation::description::*;

use std::fmt;
use std::sync::*;
use std::time::{Duration};

///
/// Represents an animation region element
///
pub struct AnimationElement {
    id:             ElementId, 
    description:    RegionDescription, 
    cached_region:  Mutex<Option<Arc<dyn AnimationRegion>>>
}

impl AnimationElement {
    ///
    /// Creates a new animation element
    ///
    pub fn new(id: ElementId, description: RegionDescription) -> AnimationElement {
        AnimationElement {
            id:             id,
            description:    description,
            cached_region:  Mutex::new(None)
        }
    }

    ///
    /// The description for this animation element
    ///
    pub fn description<'a>(&'a self) -> &'a RegionDescription {
        &self.description
    }

    ///
    /// The animation region for this animation element
    ///
    pub fn animation_region(&self) -> Arc<dyn AnimationRegion> {
        let region = {
            let mut cached_region = self.cached_region.lock().unwrap();

            if let Some(region) = &*cached_region {
                // Use the existing region
                Arc::clone(region)
            } else {
                // Cache a new region
                let region      = (&self.description).into();
                *cached_region  = Some(Arc::clone(&region));
                region
            }
        };

        region
    }
}

impl Clone for AnimationElement {
    fn clone(&self) -> Self {
        AnimationElement {
            id:             self.id,
            description:    self.description.clone(),
            cached_region:  Mutex::new(self.cached_region.lock().unwrap().clone())
        }
    }
}

impl fmt::Debug for AnimationElement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("AnimationRegion({:?}, {:?})", self.id, self.description))
    }
}

impl VectorElement for AnimationElement {
    fn id(&self) -> ElementId { 
        self.id 
    }

    fn set_id(&mut self, new_id: ElementId) { 
        self.id = new_id; 
    }

    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> { 
        None
    }

    fn render_animated(&self, gc: &mut AnimationLayerContext<'_>, properties: &VectorProperties, when: Duration) { 
        gc.add_region(self.animation_region());
        self.render_static(gc, properties, when);
    }

    fn render_static(&self, _gc: &mut dyn GraphicsContext, _properties: &VectorProperties, _when: Duration) { 
    }

    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { 
        properties 
    }

    fn control_points(&self, _properties: &VectorProperties) -> Vec<ControlPoint> { vec![] }

    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        unimplemented!()
    }
}
