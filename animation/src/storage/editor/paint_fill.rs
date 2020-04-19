use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::super::traits::*;

use flo_curves::bezier::path::algorithms::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Provides an implementaton of the fill operation
    ///
    pub (super) fn paint_fill<'a>(&'a mut self, layer_id: u64, when: Duration, path_id: ElementId, point: RawPoint, options: &'a Vec<FillOption>) -> impl 'a+Future<Output=Option<ElementWrapper>> {
        async move {
            // Fetch the brush properties
            let brush_defn_id   = self.brush_defn?;
            let brush_props_id  = self.brush_props?;

            // Decide on the fill options
            let mut gap_size    = 2.0;
            let mut step_size   = 2.0;
            let mut algorithm   = FillAlgorithm::Concave;
            let mut position    = FillPosition::Behind;

            for option in options.iter() {
                use self::FillOption::*;
                match option {
                    RayCastDistance(new_step_size)  => { step_size    = *new_step_size; }
                    MinGap(new_gap_size)            => { gap_size     = *new_gap_size; }
                    Algorithm(new_algorithm)        => { algorithm    = *new_algorithm; }
                    Position(new_position)          => { position     = *new_position; }
                }
            }

            // TODO: these options aren't implemented yet
            let _gap_size = gap_size;
            let _position = position;

            // Fetch the frame that we're going to add this fill to
            let frame = self.edit_keyframe(layer_id, when).await;
            let frame = match frame { Some(frame) => frame, None => { return None; } };

            // Generate a path element by performing the fill
            let new_path = frame.future(move |frame| {
                async move {
                    // Fetch the brush properties from the frame
                    let brush_props         = frame.elements.get(&brush_props_id).and_then(|props| props.element.clone().extract_brush_properties())?;
                    let brush_defn          = frame.elements.get(&brush_defn_id).and_then(|defn| defn.element.clone().extract_brush_definition())?;

                    // Generate a ray-casting function from the current frame
                    let ray_casting_fn      = frame.raycast(when);

                    // Set up the fill options
                    let center_point        = PathPoint::new(point.position.0, point.position.1);
                    let fill_options        = FillOptions::default();
                    let fill_options        = fill_options.with_step(step_size);

                    // Attempt to generate a path element by flood-filling
                    let fill_path           = match algorithm {
                        FillAlgorithm::Convex   => flood_fill_convex::<Path, _, _, _>(center_point, &fill_options, ray_casting_fn)?,
                        FillAlgorithm::Concave  => flood_fill_convex::<Path, _, _, _>(center_point, &fill_options, ray_casting_fn)? // TODO!!
                    };

                    // Create a new path element from the fill path we just generated
                    let path_element        = PathElement::new(path_id, fill_path, Arc::new(brush_defn), Arc::new(brush_props));
                    let element             = Vector::Path(path_element);
                    let mut wrapper         = ElementWrapper::with_element(element, when);

                    wrapper.attachments = vec![brush_props_id, brush_defn_id];

                    // Return as the result
                    Some(wrapper)
                }.boxed()
            }).await.unwrap();

            new_path
        }
    }
}