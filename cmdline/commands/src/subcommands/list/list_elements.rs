use crate::state::*;
use crate::output::*;

use flo_stream::*;
use flo_animation::*;

use futures::prelude::*;
use itertools::*;

///
/// Generates a short summary of a vector
///
fn describe_vector(vec: &Vector) -> String {
    use self::Vector::*;

    match vec {
        Transformed(transformed)        => { format!("Transformation of {}", describe_vector(&*transformed.without_transformations())) }
        BrushDefinition(_definition)    => { format!("Brush definition") }
        BrushProperties(_props)         => { format!("Brush properties") }
        BrushStroke(brush_stroke)       => { format!("Brush stroke, {} points", brush_stroke.points().len()) }
        Path(path)                      => { format!("Path, {} elements", path.path().elements().count()) }
        Motion(_motion)                 => { format!("Motion description") }
        Transformation(_transform)      => { format!("Transformation description") }
        Error                           => { format!("Error :-(") }

        Group(group)                    => { 
            let group_type  = match group.group_type() {
                GroupType::Normal => "Group",
                GroupType::Added  => "Boolean addition"
            };
            let elements    = group.elements().map(|elem| describe_vector(elem)).join(", ");

            format!("{} of ({})", group_type, elements)
        }
    }
}

///
/// Lists all of the elements in the current frame
///
pub fn list_elements<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl 'a+Future<Output=()>+Send {
    async move {
        use self::FloCommandOutput::*;

        if let Some(frame) = state.frame() {

            // List the elements in this frame
            output.publish(Message("Elements in this frame:".to_string())).await;

            let elements = frame.vector_elements().unwrap().collect::<Vec<_>>();
            for element in elements {
                // Get the ID of this element
                let element_id      = element.id().id()
                    .map(|id| format!("{:04}", id))
                    .unwrap_or_else(|| "----".to_string());

                // Create a text representation of the element
                let mut description = describe_vector(&element);
                if description.len() > 65 {
                    description.truncate(62);
                    description.extend("...".chars());
                }

                // Write it out to the output
                let element = format!("{} : {}", element_id, description);
                output.publish(Output(element)).await;
            }

        } else {

            // No frame is present
            output.publish(Error("No frame was selected".to_string())).await;
        }
    }
}
