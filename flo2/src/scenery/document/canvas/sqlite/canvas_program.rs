use super::canvas::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;
use super::super::vector_editor::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use ::serde::*;

use std::collections::{HashSet};

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
    let mut subscribers = EventSubscribers::new();

    let mut canvas  = canvas;
    let mut input   = input;
    while let Some(msg) = input.next().await {
        use SqliteCanvasRequest::*;

        match msg {
            Edit(edits) => {
                use VectorCanvas::*;

                // Track which layers and shapes are affected by this batch of edits
                let mut changed_layers = HashSet::new();
                let mut changed_shapes = HashSet::new();

                for edit in edits {
                    match edit {
                        AddLayer { new_layer_id, before_layer, }        => { changed_layers.insert(new_layer_id); canvas.add_layer(new_layer_id, before_layer).ok(); }
                        RemoveLayer(layer_id)                           => { changed_layers.insert(layer_id); canvas.remove_layer(layer_id).ok(); }
                        ReorderLayer { layer_id, before_layer, }        => { changed_layers.insert(layer_id); canvas.reorder_layer(layer_id, before_layer).ok(); }
                        AddShape(shape_id, shape_type, shape_defn)      => { changed_shapes.insert(shape_id); canvas.add_shape(shape_id, shape_type, shape_defn).ok(); }
                        RemoveShape(shape_id)                           => { changed_shapes.insert(shape_id); canvas.remove_shape(shape_id).ok(); }
                        SetShapeDefinition(shape_id, shape_defn)        => { changed_shapes.insert(shape_id); canvas.set_shape_definition(shape_id, shape_defn).ok(); }
                        AddBrush(brush_id)                              => { canvas.add_brush(brush_id).ok(); }
                        RemoveBrush(brush_id)                           => { canvas.remove_brush(brush_id).ok(); }
                        ReorderShape { shape_id, before_shape, }        => { changed_shapes.insert(shape_id); canvas.reorder_shape(shape_id, before_shape).ok(); }
                        AddShapeBrushes(shape_id, brush_ids)            => { changed_shapes.insert(shape_id); canvas.add_shape_brushes(shape_id, brush_ids).ok(); }
                        RemoveShapeBrushes(shape_id, brush_ids)         => { changed_shapes.insert(shape_id); canvas.remove_shape_brushes(shape_id, brush_ids).ok(); }
                        Subscribe(edit_target)                          => { if let Ok(edit_target) = context.send(edit_target) { subscribers.add_target(edit_target); } }

                        SetShapeParent(shape_id, parent) => {
                            changed_shapes.insert(shape_id);
                            match &parent {
                                CanvasShapeParent::Layer(layer_id)  => { changed_layers.insert(*layer_id); }
                                CanvasShapeParent::Shape(parent_id) => { changed_shapes.insert(*parent_id); }
                                CanvasShapeParent::None             => { }
                            }
                            canvas.set_shape_parent(shape_id, parent).ok();
                        }

                        SetProperty(property_target, properties) => {
                            match &property_target {
                                CanvasPropertyTarget::Layer(layer_id)    => { changed_layers.insert(*layer_id); }
                                CanvasPropertyTarget::Shape(shape_id)    => { changed_shapes.insert(*shape_id); }
                                CanvasPropertyTarget::Brush(brush_id)    => { changed_shapes.extend(canvas.shapes_with_brush(*brush_id).unwrap_or(vec![])); }
                                _                                        => { }
                            }
                            canvas.set_properties(property_target, properties).ok();
                        }

                        RemoveProperty(property_target, property_list) => {
                            match &property_target {
                                CanvasPropertyTarget::Layer(layer_id)    => { changed_layers.insert(*layer_id); }
                                CanvasPropertyTarget::Shape(shape_id)    => { changed_shapes.insert(*shape_id); }
                                CanvasPropertyTarget::Brush(brush_id)    => { changed_shapes.extend(canvas.shapes_with_brush(*brush_id).unwrap_or(vec![])); }
                                _                                        => { }
                            }
                            canvas.delete_properties(property_target, property_list).ok();
                        }
                    }
                }

                if !changed_layers.is_empty() {
                    subscribers.send(VectorCanvasUpdate::LayerChanged(changed_layers.into_iter().collect())).await;
                }
                if !changed_shapes.is_empty() {
                    subscribers.send(VectorCanvasUpdate::ShapeChanged(changed_shapes.into_iter().collect())).await;
                }
            }

            Query(query) => {
                // Query for the canvas
                use VectorQuery::*;

                match query {
                    WholeDocument(target)                                        => { todo!() },
                    DocumentOutline(target)                                      => { canvas.send_vec_query_response(target, &context, |canvas, response| canvas.query_document_outline(response)).await.ok(); },
                    Layers(target, layer_list)                                   => { canvas.send_vec_query_response(target, &context, move |canvas, response| canvas.query_layers_with_shapes(layer_list, response)).await.ok(); },
                    Shapes(target, shape_list)                                   => { canvas.send_vec_query_response(target, &context, move |canvas, response| canvas.query_shapes(shape_list, response)).await.ok(); },
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
