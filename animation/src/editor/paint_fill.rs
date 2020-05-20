use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::traits::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_curves::bezier::path::algorithms::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet};

impl StreamAnimationCore {
    ///
    /// Provides an implementaton of the fill operation
    ///
    pub (super) fn paint_fill<'a>(&'a mut self, layer_id: u64, when: Duration, path_id: ElementId, point: RawPoint, options: &'a Vec<FillOption>) -> impl 'a+Future<Output=Option<ElementWrapper>> {
        async move {
            // Fetch the brush properties
            let brush_defn_id       = self.brush_defn?;
            let brush_props_id      = self.brush_props?;

            // Decide on the fill options
            let mut gap_size        = 10.0;
            let mut step_size       = 2.0;
            let mut algorithm       = FillAlgorithm::Concave;
            let mut position        = FillPosition::Behind;
            let mut fit_precision   = 1.0;

            for option in options.iter() {
                use self::FillOption::*;
                match option {
                    RayCastDistance(new_step_size)  => { step_size      = *new_step_size; }
                    MinGap(new_gap_size)            => { gap_size       = *new_gap_size; }
                    Algorithm(new_algorithm)        => { algorithm      = *new_algorithm; }
                    Position(new_position)          => { position       = *new_position; }
                    FitPrecision(new_precision)     => { fit_precision  = *new_precision; }
                }
            }

            // TODO: these options aren't implemented yet
            let _gap_size = gap_size;
            let _position = position;

            // Fetch the frame that we're going to add this fill to
            let frame = self.edit_keyframe(layer_id, when).await;
            let frame = match frame { Some(frame) => frame, None => { return None; } };

            // Generate a path element by performing the fill
            let updates = frame.future(move |frame| {
                async move {
                    // Fetch the brush properties from the frame
                    let brush_props         = frame.elements.get(&brush_props_id).and_then(|props| props.element.clone().extract_brush_properties())?;
                    let brush_defn          = frame.elements.get(&brush_defn_id).and_then(|defn| defn.element.clone().extract_brush_definition())?;

                    // Generate a ray-casting function from the current frame
                    let ray_casting_fn      = frame.raycast(when);

                    // Set up the fill options
                    let center_point        = PathPoint::new(point.position.0, point.position.1);
                    let fill_options        = FillSettings::default();
                    let fill_options        = fill_options.with_step(step_size);
                    let fill_options        = if gap_size > 0.1 { fill_options.with_min_gap(Some(gap_size)) } else { fill_options.with_min_gap(None) };

                    // Trace the outline of the path
                    let outline             = match algorithm {
                        FillAlgorithm::Convex   => trace_outline_convex(center_point, &fill_options, move |from, to| ray_casting_fn(from, to).into_iter().map(|col| RayCollision { position: col.position, what: Some(col.what) })),
                        FillAlgorithm::Concave  => trace_outline_concave(center_point, &fill_options, ray_casting_fn)
                    };

                    // Find the element to create the path behind (if in 'behind' mode)
                    let create_behind       = match position {
                        FillPosition::InFront   => None,
                        FillPosition::Behind    => { 
                            // Get the elements that were hit in the outline
                            let outline_elements    = outline.iter().flat_map(|point| point.what).collect::<HashSet<_>>();
                            let mut create_behind   = None;

                            // Find the lowest element in the frame
                            for elem in frame.vector_elements(when) {
                                if outline_elements.contains(&elem.id()) {
                                    create_behind = Some(elem.id());
                                    break;
                                }
                            }

                            create_behind
                        }
                    };

                    // Create a path from the points in the outline
                    let curves              = fit_curve::<PathCurve>(&outline.iter().map(|point| point.position.clone()).collect::<Vec<_>>(), fit_precision)?;

                    let initial_point       = curves[0].start_point();
                    let fill_path           = Path::from_points(initial_point, curves.into_iter().map(|curve| {
                        let (cp1, cp2)  = curve.control_points();
                        let end_point   = curve.end_point();
                        (cp1, cp2, end_point)
                    }));

                    // Remove interior points from the fill path if we're using the concave algorithm
                    let fill_path            = match algorithm {
                        FillAlgorithm::Convex   => vec![fill_path],
                        FillAlgorithm::Concave  => path_remove_interior_points(&vec![fill_path], 0.01)
                    };
                    let fill_path           = Path::from_paths(fill_path.iter());

                    // Create a new path element from the fill path we just generated
                    let path_element        = PathElement::new(path_id, fill_path, Arc::new(brush_defn), Arc::new(brush_props));
                    let element             = Vector::Path(path_element);
                    let mut wrapper         = ElementWrapper::attached_with_element(element, when);

                    wrapper.attachments     = vec![brush_props_id, brush_defn_id];

                    // Edit the keyframe
                    let mut storage_updates = frame.add_element_to_end(path_id, wrapper);

                    // Move behind the 'behind' element if there is one
                    if let Some(create_behind) = create_behind.and_then(|create_behind| frame.elements.get(&create_behind)) {
                        let create_after = create_behind.order_after;
                        storage_updates.extend(frame.order_after(path_id, None, create_after));
                    }

                    // Result is 'no wrapper', as we add it ourselves
                    Some(storage_updates)
                }.boxed()
            }).await.unwrap();

            // Send the updates to storage
            self.request(updates.unwrap_or_else(|| PendingStorageChange::new())).await;

            None
        }
    }
}
