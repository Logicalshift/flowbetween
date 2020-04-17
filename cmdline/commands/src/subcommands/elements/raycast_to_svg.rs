use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_curves::debug::*;
use flo_curves::bezier::path::*;
use flo_stream::*;
use flo_animation::*;

use futures::prelude::*;

use std::sync::*;

///
/// Outputs a series of SVG files that show raycasting on the specified element
///
pub fn raycast_to_svg<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState, element_id: ElementId) -> impl 'a+Future<Output=Result<(), CommandError>>+Send {
    async move {
        // We need to have the frame with the element selected
        let frame       = state.frame();
        let frame       = match frame {
            Some(frame) => frame,
            None        => { return Err(CommandError::NoFrameSelected) }
        };

        // Fetch the element
        let element     = frame.element_with_id(element_id);
        let element     = match element {
            Some(element)   => element,
            None            => { return Err(CommandError::ElementNotFound(element_id)) }
        };

        // Fetch the paths for this element
        let paths = match &element {
            Vector::Group(group)  => { 
                let properties  = frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));
                group.elements()
                    .map(|elem| elem.to_path(&properties, PathConversion::Fastest))
                    .flatten()
                    .collect::<Vec<_>>()
            },

            element => {
                let properties = frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));
                element.to_path(&properties, PathConversion::Fastest).map(|path| vec![path]).unwrap_or(vec![])
            }
        };

        output.publish(FloCommandOutput::Message(format!("Adding {} paths", paths.len()))).await;

        // The way grouping works is to remove interior points and then combine the paths with a rule, we'll simulate that here and
        // use flo_curve's debugging function to generate a set of SVG files
        //let current_path = None;

        for (path_num, path) in paths.into_iter().enumerate() {
            // Start writing the 'remove interior points' file
            let remove_interior_filename = format!("remove_interior_{}.svg", path_num);
            output.publish(FloCommandOutput::Message(format!("  Writing {}", remove_interior_filename))).await;
            output.publish(FloCommandOutput::BeginOutput(remove_interior_filename)).await;
            output.publish(FloCommandOutput::Output("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>
                <!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">
                <svg width=\"100%\" height=\"100%\" viewBox=\"0 0 2000 4000\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" xml:space=\"preserve\" style=\"fill-rule:evenodd;clip-rule:evenodd;stroke-linecap:round;stroke-miterlimit:8;\">".to_string())).await;

            // Generate a graph from this path
            let mut remove_interior = GraphPath::new();
            remove_interior         = remove_interior.merge(GraphPath::from_merged_paths(path.iter().map(|sub_path| (sub_path, PathLabel(0, PathDirection::from(sub_path))))));

            // Self-collide to generate the 'remove interior points' status
            remove_interior.self_collide(0.01);
            remove_interior.round(0.01);

            // Set the exterior edges using the 'add' algorithm
            remove_interior.set_exterior_by_removing_interior_points();

            // Finish writing the SVG (TODO: get rays?)
            let svg = graph_path_svg_string(&remove_interior, vec![]);
            output.publish(FloCommandOutput::Output(svg)).await;
            output.publish(FloCommandOutput::Output("\n</svg>\n".to_string())).await;

            // Gaps are healed after we write the initial raycast
            remove_interior.heal_exterior_gaps();
        }

        Ok(())
    }
}
