use super::traits::*;

use flo_curves::*;
use flo_curves::bezier::path::*;
use flo_canvas::*;

use futures::*;

use std::iter;
use std::fmt::*;
use std::sync::*;
use std::time::Duration;

fn svg_path_string<Path: BezierPath>(path: &Path) -> String 
where Path::Point: Coordinate2D {
    let mut svg = String::new();

    write!(&mut svg, "M {} {}", path.start_point().x(), path.start_point().y());
    for (cp1, cp2, end) in path.points() {
        write!(&mut svg, " C {} {}, {} {}, {} {}", cp1.x(), cp1.y(), cp2.x(), cp2.y(), end.x(), end.y());
    }

    svg
}

///
/// Computes or retrieves the onion skin for a particular layer at a specified time.specified
/// 
/// This is the set of Move/Line/Bezier curve elements forming the path for this onion skin. The `NewPath` and any actual
/// drawing instructions are left out of the list generated here, so the onion skin can be rendered any way that is needed.
///
pub fn onion_skin_for_layer(layer: Arc<dyn Layer>, when: Duration) -> CacheProcess<Arc<Vec<Draw>>, Box<dyn Future<Item=Arc<Vec<Draw>>, Error=Canceled>+Send>> {
    layer.get_canvas_cache_at_time(when)
        .retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(move || {
            // Fetch the elements for the frame
            let frame                       = layer.get_frame_at_time(when);
            let elements                    = frame.vector_elements().unwrap_or(Box::new(vec![].into_iter()));

            let mut active_attachments      = vec![];
            let mut properties              = Arc::new(VectorProperties::default());
            let mut onion_skin: Vec<Path>   = vec![];

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
                    for element_id in active_attachments.iter() {
                        if let Some(attach_element) = frame.element_with_id(element_id.clone()) {
                            properties = attach_element.update_properties(Arc::clone(&properties));
                        }
                    }
                }

                // Add this element to the onion skin path
                if let Some(element_path) = element.to_path(&properties) {
                    if element_path.len() > 0 && (element_path.len() != 1 || element_path[0].len() > 0) {
                        let element_path_without_interior = path_remove_interior_points::<_, _, Path>(&element_path, 0.01);
                        if element_path_without_interior.len() == 0 {
                            println!("Remove interior points removed all points? {:?} {:?}", element_path.len(), element_path);
                            for p in element_path {
                                println!("  {:?}", svg_path_string(&p));
                            }
                        } else {
                            match (*properties).brush.drawing_style() {
                                BrushDrawingStyle::Draw     => { onion_skin = path_add(&onion_skin, &element_path_without_interior, 0.01); }
                                BrushDrawingStyle::Erase    => { onion_skin = path_sub(&onion_skin, &element_path_without_interior, 0.01); }
                            }
                        }
                    }
                }
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
