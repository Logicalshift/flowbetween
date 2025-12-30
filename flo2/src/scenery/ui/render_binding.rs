use flo_binding::*;
use flo_draw::canvas::scenery::*;
use flo_draw::canvas::*;
use flo_scene::*;
use flo_scene_binding::*;

use futures::prelude::*;

use std::sync::*;

///
/// The render binding subprogram takes a binding to some render instructions, a namespace and a layer, and
/// generates drawing requests for that layer whenever that binding is updated. Optionally it can take the 
/// identifier of a parent program, and stop and clear the layer if that program ever quits.
///
/// Rendering is done in scene idle cycles: this ensures that if many updates arrive only a single drawing
/// event will be generated.
///
/// This subprogram is very useful for drawing dynamically as it removes the need for a program to generate
/// its own rendering events.
///
pub fn render_binding_program(input: InputStream<BindingProgram>, context: SceneContext, render_layer: (NamespaceId, LayerId), parent_program: Option<SubProgramId>, rendering: impl Send + Into<BindRef<Vec<Draw>>>) -> impl Send + Future<Output=()> {
    async move {
        // Action is to clear the layer and perform the rendering instructions specified by the binding
        let mut action = BindingAction::new(move |new_drawing: Vec<Draw>, context| {
            let context = context.clone();

            async move {
                let mut new_drawing = new_drawing;

                // Set up to use the specified layer, and pop the state when we're done
                new_drawing.splice(0..0, vec![
                    Draw::PushState,
                    Draw::Namespace(render_layer.0),
                    Draw::Layer(render_layer.1),
                    Draw::ClearLayer,
                ]);
                new_drawing.push(Draw::PopState);

                // Send the request to the renderer
                context.send_message(DrawingRequest::Draw(Arc::new(new_drawing))).await.ok();
            }
        });

        if let Some(parent_program) = parent_program { 
            action = action.with_parent_program(parent_program);
        }

        // When the parent program stops, clear the layer that we were rendering
        action = action.with_stop_action(move |context| async move {
            context.send_message(DrawingRequest::Draw(Arc::new(vec![
                Draw::PushState,
                Draw::Namespace(render_layer.0),
                Draw::Layer(render_layer.1),
                Draw::ClearLayer,
                Draw::PopState,
            ]))).await.ok();
        }.boxed());

        // Run as a binding program
        binding_program(input, context.clone(), rendering, action).await;
    }
}
