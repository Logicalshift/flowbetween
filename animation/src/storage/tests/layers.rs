use super::*;

use flo_stream::*;
use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Creates path components for a circular path
///
fn circle_path(pos: (f64, f64), radius: f64) -> Arc<Vec<PathComponent>> {
    let mut drawing = vec![];

    drawing.new_path();
    drawing.circle(pos.0 as _, pos.1 as _, radius as _);

    let path        = Path::from_drawing(drawing);

    Arc::new(path.elements().collect())
}

#[test]
fn add_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 1);
}

#[test]
fn add_multiple_layers() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(3),
        AnimationEdit::AddNewLayer(4)
    ]);

    assert!(anim.get_layer_ids().len() == 3);
}

#[test]
fn remove_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 1);

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 0);
}

#[test]
fn remove_nonexistent_layer() {
    let anim = create_animation();

    assert!(anim.get_layer_ids().len() == 0);

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);

    assert!(anim.get_layer_ids().len() == 0);
}

#[test]
fn single_layer_id() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 1);
    assert!(layer_ids == vec![2]);

}

#[test]
fn retrieve_layer_ids() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::AddNewLayer(42)
    ]);

    let mut layer_ids = anim.get_layer_ids();
    layer_ids.sort();
    assert!(layer_ids.len() == 2);
    assert!(layer_ids == vec![2, 42]);
}

#[test]
fn retrieve_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());
    assert!(layer.unwrap().id() == 2);
}

#[test]
fn non_existent_layer() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2)
    ]);

    let layer = anim.get_layer_with_id(3);
    assert!(layer.is_none());
}

#[test]
fn add_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe2() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn add_keyframe_with_layer_editor() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
    ]);

    executor::block_on(async {
        let mut sink = anim.edit();

        sink.publish(Arc::new(vec![AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))])).await;
        sink.when_empty().await;
    });


    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    let keyframes: Vec<_> = layer.unwrap().get_key_frames().collect();
    assert!(keyframes.len() == 1);
    assert!(keyframes[0] == Duration::from_millis(250));
}

#[test]
fn remove_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::RemoveKeyFrame(Duration::from_millis(250)))
    ]);

}

#[test]
fn retrieve_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250)))
    ]);
}

#[test]
fn find_previous_and_next_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(500))),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(750)))
    ]);


    let layer = anim.get_layer_with_id(2).unwrap();

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(375));
    assert!(previous == Some(Duration::from_millis(250)));
    assert!(next == Some(Duration::from_millis(500)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(625));
    assert!(previous == Some(Duration::from_millis(500)));
    assert!(next == Some(Duration::from_millis(750)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(1000));
    assert!(previous == Some(Duration::from_millis(750)));
    assert!(next == None);

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(0));
    assert!(previous == None);
    assert!(next == Some(Duration::from_millis(250)));

    let (previous, next) = layer.previous_and_next_key_frame(Duration::from_millis(500));
    assert!(previous == Some(Duration::from_millis(250)));
    assert!(next == Some(Duration::from_millis(750)));
}

#[test]
fn remove_layer_with_keyframe() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(250))),
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_some());

    anim.perform_edits(vec![
        AnimationEdit::RemoveLayer(2)
    ]);

    let layer = anim.get_layer_with_id(2);
    assert!(layer.is_none());
}

#[test]
fn delete_layer_after_drawing_brush_stroke() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    anim.perform_edits(vec![AnimationEdit::RemoveLayer(2)]);
}

#[test]
fn get_active_brush() {
    let create_layer = "
        +B
        LB+tAAAAAA";

    let draw_lines = "
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAiB+2FAAodjLHRF9PA8BAcNj5P1EA4AAAAAAAAAAAAAAGAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAANAAAAAAAAAAAAAAEAAAAAAAAlXAaIAEAAAAAAAA2MAsBACAAAAAAAAXPAGCACAAA8PAAArlAbEACAAAAAAAAGWAUCADAAAAAAAAbYA5BABAAAAAAAAVaArBABAAAAAAAAocAsBABAAAAAAAAieAQBABAAAAAAAAOgADBAAAAAAAAAA5hA1AAAAAAAAAAAXjAbAAAAAA4PAAAM3Cs9PAAAAAAAAAAkAU+PAAAA8PAAA8iAI+PAAAAAAAIAfhAU+PAAAAAAAAAyfAU+PAAAA4PAAADdAj+PAAAAAAAAADxA28P//PAAAAAAmvAv6P8/PA8PAAAQJAw+P0/PAAAAAA9GAi+P4/PA4PAAAbEA9+P3/PAAAAAAhCA9+Pw/PAAAAAA1AAL/Pn/PAAAAAAAAAl/PZ/PAAAAAA69PAAAF/PAAAAAA
        LBPtAAAAAAS+DAjBAAoZmS0QAA4MzFIRt9PAsBAYNJBAB/PQAAAAAAAAAAAAAAIAAAAAAAAAAAAAADAAAAAAAAAAAAAABAAAAAAAAAAAAAACAAAAAAAAAAAAAALAAAAAAAAAAAAAAEAAAAAAAAoAAbkPDAAAAAAAANAAUyPCAAAAAAAAAAALvPCAAAAAAAAAAAKrPCAAAAAAEAi+P9mPCAAAAAAAAmzPkbOBAAAAAAAAU6PNYPDAAAAAAIA65PiWPAAAAAAAEAU6PhWPAAAAAAAAAm7PYXPAAAAAAAEAD9PEZPAAAAAAAAA9+PYbPAAAAAAAIAAAA6dPAAAAAAAAAsBA8CPAAAAAAAAAUCAflPAAAAAAAEAhCAAoPAAAAAAAAAGCAhqPAAAAAAAIAHCA3sPAAAAAAAAA4BAKvPAAAAAAAAAeBAexPAAAAAAAEAeBAYzPAAAAAAAAAQBAs1P//PAAAAAAXDA6tP+/PAAAAAADBAAAAu/PAAAAEAQBAhCA0/PAAAAAA2AAXDAq/PAAAAAAQBAGKAE/PAAAAAA
    ";

    // Create a layer
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, create_layer);

    // As there are no brush strokes in the layer yet, there should be no active brush
    let layer           = animation.get_layer_with_id(1).unwrap();
    let active_brush    = layer.as_vector_layer().unwrap().active_brush(Duration::from_millis(0));

    assert!(active_brush.is_none());

    // Draw some lines in the layer
    perform_serialized_edits(&mut animation, draw_lines);

    // Should now be an active brush
    let layer           = animation.get_layer_with_id(1).unwrap();
    let active_brush    = layer.as_vector_layer().unwrap().active_brush(Duration::from_millis(0));

    assert!(active_brush.is_some());
}

#[test]
fn delete_last_item() {
    let anim = create_animation();

    // Create a square
    let square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(100.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 100.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::SelectBrush(
                ElementId::Assigned(1),
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            ))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(2), BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), square.clone())))
    ]);

    // Layer should contain the square
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);

    // Delete the square, and add it back twice
    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(3)], ElementEdit::Delete),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(4), square.clone()))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(5), square.clone())))
    ]);

    // Layer should contain 2 groups with the inside and outside elements in it
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() != 1);
    assert!(elements.len() == 2);

    assert!(elements[0].id() == ElementId::Assigned(4));
    assert!(elements[1].id() == ElementId::Assigned(5));
}

#[test]
fn reorder_1() {
    use AnimationEdit::*;
    use LayerEdit::*;

    let anim = create_animation();

    anim.perform_edits(vec![
        AddNewLayer(0),
        Layer(0, AddKeyFrame(Duration::from_millis(0))),
        AddNewLayer(1),
        Layer(1, AddKeyFrame(Duration::from_millis(0))),
        AddNewLayer(2),
        Layer(2, AddKeyFrame(Duration::from_millis(0))),

        Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

        Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
        Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
        Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

        Layer(1, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(102), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        Layer(1, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(103), BrushProperties::new()))),

        Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), circle_path((100.0, 100.0), 50.0)))),
        Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(4), circle_path((100.0, 150.0), 50.0)))),
        Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(5), circle_path((100.0, 200.0), 50.0)))),

        Layer(2, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(104), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        Layer(2, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(105), BrushProperties::new()))),

        Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(6), circle_path((100.0, 100.0), 50.0)))),
        Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(7), circle_path((100.0, 150.0), 50.0)))),
        Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(8), circle_path((100.0, 200.0), 50.0)))),
    ]);

    let layers = anim.get_layer_ids();
    println!("{:?}", layers);
    assert!(layers == vec![0, 1, 2]);

    anim.perform_edits(vec![
        Layer(2, SetOrdering(0))
    ]);

    let layers = anim.get_layer_ids();
    println!("{:?}", layers);
    assert!(layers == vec![2, 0, 1]);
}
