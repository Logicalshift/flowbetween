use super::traits::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use futures::future::{BoxFuture};

use std::iter;
use std::sync::*;
use std::time::Duration;

///
/// Computes or retrieves the onion skin for a particular layer at a specified time.specified
///
/// This is the set of Move/Line/Bezier curve elements forming the path for this onion skin. The `NewPath` and any actual
/// drawing instructions are left out of the list generated here, so the onion skin can be rendered any way that is needed.
///
pub fn onion_skin_for_layer(layer: Arc<dyn Layer>, when: Duration) -> CacheProcess<Arc<Vec<Draw>>, BoxFuture<'static, Arc<Vec<Draw>>>> {
    layer.get_canvas_cache_at_time(when)
        .retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(move || {
            // Fetch the elements for the frame
            let frame                       = layer.get_frame_at_time(when);
            let elements                    = frame.vector_elements().unwrap_or(Box::new(vec![].into_iter()));

            let mut active_attachments      = vec![];
            let mut properties              = Arc::new(VectorProperties::default());
            let mut onion_skin: Vec<Path>   = vec![];
            let mut waiting_to_add          = vec![];

            // Generate the onion skin path for this frame
            for element in elements {
                // Fetch the attachment IDs
                let element_attachments = frame.attached_elements(element.id()).into_iter().map(|(id, _type)| id).collect::<Vec<_>>();

                // Update the properties based on the attachments, if the attachments are different
                if active_attachments != element_attachments {
                    // These attachments are active now
                    active_attachments = element_attachments;

                    // Apply them to generate the properties for the following elements
                    properties = Arc::new(VectorProperties::default());
                    properties = frame.apply_properties_for_element(&element, properties);
                }

                // Add this element to the onion skin path
                if let Some(element_path) = element.to_path(&properties, PathConversion::RemoveInteriorPoints) {
                    if element_path.len() > 0 && (element_path.len() != 1 || element_path[0].len() > 0) {
                        match (*properties).brush.drawing_style() {
                            BrushDrawingStyle::Draw     => { waiting_to_add.push(element_path); }
                            BrushDrawingStyle::Erase    => {
                                if waiting_to_add.len() > 0 {
                                    waiting_to_add.insert(0, onion_skin);
                                    onion_skin = path_add_chain(&waiting_to_add, 0.01);
                                    waiting_to_add.clear();
                                }
                                onion_skin = path_sub(&onion_skin, &element_path, 0.01);
                            }
                        }
                    }
                }
            }

            // Finish adding any paths that are still waiting
            if waiting_to_add.len() > 0 {
                waiting_to_add.insert(0, onion_skin);
                onion_skin = path_add_chain(&waiting_to_add, 0.01);
                waiting_to_add.clear();
            }

            // Convert to a series of drawing instructions
            Arc::new(onion_skin.into_iter()
                .flat_map(|path| {
                    let start_point = path.start_point();

                    iter::once(Draw::Move(start_point.x(), start_point.y()))
                        .chain(path.points().map(|(cp1, cp2, end)| Draw::BezierCurve((end.x(), end.y()), (cp1.x(), cp1.y()), (cp2.x(), cp2.y()))))
                        .chain(iter::once(Draw::ClosePath))
                })
                .collect())
        }))
}
