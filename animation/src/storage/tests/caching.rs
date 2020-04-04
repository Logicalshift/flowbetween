use super::*;

use flo_canvas::*;

use futures::executor;
use std::time::Duration;
use std::sync::*;

#[test]
fn set_and_retrieve_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == Some(Arc::new(vec![Draw::NewPath, Draw::Fill])));
}

#[test]
fn retrieve_or_generate_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    let cached_drawing  = cache.retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(|| Arc::new(vec![Draw::NewPath, Draw::Fill])));

    // Should initially be a future indicating the cached item will be generated eventually
    assert!(match cached_drawing { CacheProcess::Process(_) => true, _ => false });

    // ... and eventually evaluate to the drawing we specified in the generate function
    let cached_drawing = executor::block_on(cached_drawing);

    assert!(cached_drawing == Arc::new(vec![Draw::NewPath, Draw::Fill]));

    // Should be able to retrieve instantly next time
    let cached_drawing  = cache.retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(|| Arc::new(vec![Draw::NewPath, Draw::Fill])));

    assert!(match cached_drawing { CacheProcess::Cached(_) => true, _ => false });
    assert!(match cached_drawing { CacheProcess::Cached(cached_drawing) => cached_drawing == Arc::new(vec![Draw::NewPath, Draw::Fill]), _ => false });
}

#[test]
fn invalidate_cached_onionskin() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));
    cache.invalidate(CacheType::OnionSkinLayer);

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == None);
}

#[test]
fn retrieve_cached_onionskin_from_different_time() {
    let anim = create_animation();

    anim.perform_edits(vec![AnimationEdit::AddNewLayer(24)]);

    let layer           = anim.get_layer_with_id(24).unwrap();
    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(2000));

    cache.store(CacheType::OnionSkinLayer, Arc::new(vec![Draw::NewPath, Draw::Fill]));
    cache.invalidate(CacheType::OnionSkinLayer);

    let cache           = layer.get_canvas_cache_at_time(Duration::from_millis(1500));
    let cached_drawing  = cache.retrieve(CacheType::OnionSkinLayer);

    assert!(cached_drawing == None);
}
