use flo_canvas::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::*;
use flo_canvas_animation::effects::*;

use futures::prelude::*;
use futures::executor;
use futures::stream;

use std::time::{Duration};

#[test]
pub fn include_path_in_region() {
    // Create an animation layer
    let mut animation_layer = AnimationLayer::new();

    // Draw 4 shapes, two at two different times
    let circle1             = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();
    let circle2             = Circle::new(Coord2(300.0, 100.0), 50.0).to_path::<SimpleBezierPath>();

    let mut draw1           = vec![];
    let mut draw2           = vec![];

    draw1.new_path();
    draw1.bezier_path(&circle1);
    draw1.fill();

    draw2.new_path();
    draw2.bezier_path(&circle1);
    draw2.fill();

    animation_layer.set_time(Duration::from_millis(0));
    animation_layer.draw(draw1.iter().cloned());
    animation_layer.draw(draw2.iter().cloned());

    animation_layer.set_time(Duration::from_millis(1000));
    animation_layer.draw(draw1.iter().cloned());
    animation_layer.draw(draw2.iter().cloned());

    // Should be two shapes initially and four shapes later on
    let at_time_zero    = executor::block_on(async { 
        drawing_to_paths::<SimpleBezierPath, _>(stream::iter(
            animation_layer.render_at_time(Duration::from_millis(0)).await.into_iter()
        )).collect::<Vec<_>>().await
    });
    let at_time_later   = executor::block_on(async { 
        drawing_to_paths::<SimpleBezierPath, _>(stream::iter(
            animation_layer.render_at_time(Duration::from_millis(1000)).await.into_iter()
        )).collect::<Vec<_>>().await
    });

    assert!(at_time_zero.len() == 2);
    assert!(at_time_later.len() == 4);

    // Create two animation regions that enclose the shapes. Use the stop-motion effect so we replace the paths instead of adding them
    let region1         = Circle::new(Coord2(100.0, 100.0), 60.0).to_path::<SimpleBezierPath>();
    let region2         = Circle::new(Coord2(300.0, 100.0), 60.0).to_path::<SimpleBezierPath>();
    let region1         = FrameByFrameEffect::ReplaceWhole.with_region(vec![region1]);
    let region2         = FrameByFrameEffect::ReplaceWhole.with_region(vec![region2]);
    animation_layer.add_region(region1);
    animation_layer.add_region(region2);

    let at_time_zero    = executor::block_on(async { 
        drawing_to_paths::<SimpleBezierPath, _>(stream::iter(
            animation_layer.render_at_time(Duration::from_millis(0)).await.into_iter()
        )).collect::<Vec<_>>().await
    });
    let at_time_later   = executor::block_on(async { 
        drawing_to_paths::<SimpleBezierPath, _>(stream::iter(
            animation_layer.render_at_time(Duration::from_millis(1000)).await.into_iter()
        )).collect::<Vec<_>>().await
    });

    assert!(at_time_zero.len() == 2);
    assert!(at_time_later.len() == 2);
}
