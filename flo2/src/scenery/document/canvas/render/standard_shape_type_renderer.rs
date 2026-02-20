use super::shape_type_renderer::*;
use super::super::basic_properties::*;
use super::super::property::*;

use flo_draw::canvas::*;
use flo_scene::*;

///
/// Runs the standard shape renderer program (for `ShapeType::default().render_program_id()`)
///
/// This renders shapes using the properties defined in basic_properties. Groups are just rendered in a 'straight through' fashion
///
pub async fn standard_shape_type_renderer_program(input: InputStream<RenderShapesRequest>, context: SceneContext) {
    shape_renderer_program(input, context, |shape, drawing| {
        // Generate the path
        drawing.new_path();
        shape.shape.to_path()
            .into_iter()
            .for_each(|path| drawing.bezier_path(&path));

        // Set up the properties to render it
        let fill    = FlatFill::from_properties(shape.properties.iter());
        let stroke  = Stroke::from_properties(shape.properties.iter());

        if let Some(fill) = fill {
            fill.draw(drawing);
        }

        if let Some(stroke) = stroke {
            stroke.draw(drawing);
        }

        // Any grouped shapes are rendered after
        shape.group.iter()
            .for_each(|(_shape, shape_drawing)| drawing.extend(shape_drawing.iter().cloned()));
    }).await;
}
