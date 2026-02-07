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
    Edit(Vec<VectorCanvas>),
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
            Edit(edits) => {
                for edit in edits {
                    use VectorCanvas::*;
                    match edit {
                        AddLayer { new_layer_id, before_layer, }        => { canvas.add_layer(new_layer_id, before_layer).ok(); }
                        RemoveLayer(layer_id)                           => { canvas.remove_layer(layer_id).ok(); }
                        ReorderLayer { layer_id, before_layer, }        => { canvas.reorder_layer(layer_id, before_layer).ok(); }
                        AddShape(shape_id, shape_defn)                  => { canvas.add_shape(shape_id, shape_defn).ok(); }
                        RemoveShape(shape_id)                           => { canvas.remove_shape(shape_id).ok(); }
                        SetShapeDefinition(shape_id, shape_defn)        => { canvas.set_shape_definition(shape_id, shape_defn).ok(); }
                        AddBrush(brush_id)                              => { canvas.add_brush(brush_id).ok(); }
                        RemoveBrush(brush_id)                           => { canvas.remove_brush(brush_id).ok(); }
                        ReorderShape { shape_id, before_shape, }        => { canvas.reorder_shape(shape_id, before_shape).ok(); }
                        SetShapeParent(shape_id, parent)                => { canvas.set_shape_parent(shape_id, parent).ok(); }
                        SetProperty(property_target, properties)        => { canvas.set_properties(property_target, properties).ok(); }
                        AddShapeBrushes(shape_id, brush_ids)            => { canvas.add_shape_brushes(shape_id, brush_ids).ok(); }
                        RemoveProperty(property_target, property_list)  => { todo!() }
                        RemoveShapeBrushes(shape_id, brush_ids)         => { canvas.remove_shape_brushes(shape_id, brush_ids).ok(); }
                        Subscribe(edit_target)                          => { todo!() }
                    }
                }
            }

            Query(query) => {
                use VectorQuery::*;
                match query {
                    WholeDocument(target)                                        => { todo!() },
                    DocumentOutline(target)                                      => { canvas.send_vec_query_response(target, &context, |canvas, response| canvas.query_document_outline(response)).await.ok(); },
                    Layers(target, layer_list)                                   => { todo!() },
                    Shapes(target, shape_list)                                   => { todo!() },
                    Brushes(target, brush_list)                                  => { todo!() },
                    ShapesInRegion { target, search_layers, region, inclusive, } => { todo!() },
                    ShapesAtPoint { target, search_layers, point, }              => { todo!() },
                }
            }
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

        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.ready_chunks(100).map(|msgs| SqliteCanvasRequest::Edit(msgs)))), (), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg)))), (), StreamId::with_message_type::<VectorQuery>()).unwrap();

        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.ready_chunks(100).map(|msgs| SqliteCanvasRequest::Edit(msgs))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorCanvas>()).unwrap();
        init_context.connect_programs((), StreamTarget::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|msg| SqliteCanvasRequest::Query(msg))), SubProgramId::called("flowbetween::sqlite_canvas")), StreamId::with_message_type::<VectorQuery>()).unwrap();
    }
}
