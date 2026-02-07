use super::canvas::*;
use super::super::queries::*;
use super::super::vector_editor::*;

use flo_scene::*;

use futures::prelude::*;
use ::serde::*;

///
/// Messages for the sqlite canvas program
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SqliteCanvasRequest {
    Edit(VectorCanvas),
    Query(VectorQuery),
}

///
/// Runs a program that edits the document stored in the Sqlite connection
///
pub async fn sqlite_canvas_program(input: InputStream<SqliteCanvasRequest>, context: SceneContext, canvas: SqliteCanvas) {
    let mut canvas  = canvas;
    let mut input   = input;
    while let Some(msg) = input.next().await {
        use SqliteCanvasRequest::*;
        use VectorCanvas::*;
        use VectorQuery::*;

        match msg {
            Edit(AddLayer { new_layer_id, before_layer, })          => { canvas.add_layer(new_layer_id, before_layer).ok(); }
            Edit(RemoveLayer(layer_id))                             => { canvas.remove_layer(layer_id).ok(); }
            Edit(ReorderLayer { layer_id, before_layer, })          => { canvas.reorder_layer(layer_id, before_layer).ok(); }
            Edit(AddShape(shape_id, shape_defn))                    => { canvas.add_shape(shape_id, shape_defn).ok(); }
            Edit(RemoveShape(shape_id))                             => { canvas.remove_shape(shape_id).ok(); }
            Edit(SetShapeDefinition(shape_id, shape_defn))          => { canvas.set_shape_definition(shape_id, shape_defn).ok(); }
            Edit(AddBrush(brush_id))                                => { todo!() }
            Edit(RemoveBrush(brush_id))                             => { todo!() }
            Edit(ReorderShape { shape_id, before_shape, })          => { canvas.reorder_shape(shape_id, before_shape).ok(); }
            Edit(SetShapeParent(shape_id, parent))                  => { canvas.set_shape_parent(shape_id, parent).ok(); }
            Edit(SetProperty(property_target, properties))          => { canvas.set_properties(property_target, properties).ok(); }
            Edit(AddShapeBrushes(shape_id, brush_ids))              => { canvas.add_shape_brushes(shape_id, brush_ids).ok(); }
            Edit(RemoveProperty(property_target, property_list))    => { todo!() }
            Edit(RemoveShapeBrushes(shape_id, brush_ids))           => { canvas.remove_shape_brushes(shape_id, brush_ids).ok(); }

            Edit(Subscribe(edit_target))                            => { todo!() }

            Query(WholeDocument(target))                                        => { todo!() },
            Query(DocumentOutline(target))                                      => { canvas.send_vec_query_response(target, &context, |canvas, response| canvas.query_document_outline(response)).await.ok(); },
            Query(Layers(target, layer_list))                                   => { todo!() },
            Query(Shapes(target, shape_list))                                   => { todo!() },
            Query(Brushes(target, brush_list))                                  => { todo!() },
            Query(ShapesInRegion { target, search_layers, region, inclusive, }) => { todo!() },
            Query(ShapesAtPoint { target, search_layers, point, })              => { todo!() },
        }
    }
}

///
/// Edits a blank document in memory
///
pub async fn sqlite_canvas_program_new_in_memory(input: InputStream<SqliteCanvasRequest>, context: SceneContext) {
    let canvas = SqliteCanvas::new_in_memory().unwrap();

    sqlite_canvas_program(input, context, canvas).await;
}

impl SceneMessage for SqliteCanvasRequest {
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::sqlite_canvas").into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.add_subprogram(SubProgramId::called("flowbetween::sqlite_canvas"), sqlite_canvas_program_new_in_memory, 20);

        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Edit(msg)))), (), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg)))), (), StreamId::with_message_type::<VectorQuery>()).unwrap();

        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Edit(msg))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorQuery>()).unwrap();
    }
}
