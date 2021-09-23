use flo_canvas::*;
use flo_canvas_animation::*;

use std::sync::*;
use std::time::{Duration};

///
/// Creates a sample animation path which can be identified in the future by its ID number
///
fn create_path(id_num: usize) -> AnimationPath {
    AnimationPath {
        appearance_time:    Duration::from_millis(0),
        attributes:         AnimationPathAttribute::Fill(BlendMode::SourceOver, Color::Rgba(0.0, 0.0, 0.0, 1.0), WindingRule::EvenOdd),
        path:               Arc::new(vec![(Coord2(id_num as _, id_num as _), vec![])])
    }
}

///
/// True if the specified path was generated with a particular ID number
///
fn is_path_id(path: &AnimationPath, id_num: usize) -> bool {
    if path.path.len() != 1 {
        false
    } else {
        let (Coord2(x, _y), _points) = &path.path[0];
        let distance = ((id_num as f64) - x).abs();
        distance < 0.01
    }
}

#[test]
fn iterate_without_extra_content() {
    let paths                   = vec![
        create_path(1),
        create_path(2),
        create_path(3)
    ];
    let content                 = AnimationRegionContent::from_paths(paths);

    let mut content_iterator    = content.paths();

    assert!(is_path_id(content_iterator.next().unwrap(), 1));
    assert!(is_path_id(content_iterator.next().unwrap(), 2));
    assert!(is_path_id(content_iterator.next().unwrap(), 3));
    assert!(content_iterator.next().is_none());
}

#[test]
fn iterate_with_prefix() {
    let paths                   = vec![
        create_path(1),
        create_path(2),
        create_path(3)
    ];
    let prefix_paths            = vec![
        create_path(4),
        create_path(5),
        create_path(6)
    ];
    let prefix                  = AnimationRegionContent::from_paths(prefix_paths);
    let content                 = AnimationRegionContent::from_paths(paths).with_prefix(Arc::new(prefix));

    let mut content_iterator    = content.paths();

    assert!(is_path_id(content_iterator.next().unwrap(), 4));
    assert!(is_path_id(content_iterator.next().unwrap(), 5));
    assert!(is_path_id(content_iterator.next().unwrap(), 6));

    assert!(is_path_id(content_iterator.next().unwrap(), 1));
    assert!(is_path_id(content_iterator.next().unwrap(), 2));
    assert!(is_path_id(content_iterator.next().unwrap(), 3));

    assert!(content_iterator.next().is_none());
}

#[test]
fn iterate_with_suffix() {
    let paths                   = vec![
        create_path(1),
        create_path(2),
        create_path(3)
    ];
    let suffix_paths            = vec![
        create_path(7),
        create_path(8),
        create_path(9)
    ];
    let suffix                  = AnimationRegionContent::from_paths(suffix_paths);
    let content                 = AnimationRegionContent::from_paths(paths).with_suffix(Arc::new(suffix));

    let mut content_iterator    = content.paths();

    assert!(is_path_id(content_iterator.next().unwrap(), 1));
    assert!(is_path_id(content_iterator.next().unwrap(), 2));
    assert!(is_path_id(content_iterator.next().unwrap(), 3));

    assert!(is_path_id(content_iterator.next().unwrap(), 7));
    assert!(is_path_id(content_iterator.next().unwrap(), 8));
    assert!(is_path_id(content_iterator.next().unwrap(), 9));

    assert!(content_iterator.next().is_none());
}

#[test]
fn iterate_with_prefix_and_suffix() {
    let paths                   = vec![
        create_path(1),
        create_path(2),
        create_path(3)
    ];
    let prefix_paths            = vec![
        create_path(4),
        create_path(5),
        create_path(6)
    ];
    let suffix_paths            = vec![
        create_path(7),
        create_path(8),
        create_path(9)
    ];
    let prefix                  = AnimationRegionContent::from_paths(prefix_paths);
    let suffix                  = AnimationRegionContent::from_paths(suffix_paths);
    let content                 = AnimationRegionContent::from_paths(paths).with_prefix(Arc::new(prefix)).with_suffix(Arc::new(suffix));

    let mut content_iterator    = content.paths();

    assert!(is_path_id(content_iterator.next().unwrap(), 4));
    assert!(is_path_id(content_iterator.next().unwrap(), 5));
    assert!(is_path_id(content_iterator.next().unwrap(), 6));

    assert!(is_path_id(content_iterator.next().unwrap(), 1));
    assert!(is_path_id(content_iterator.next().unwrap(), 2));
    assert!(is_path_id(content_iterator.next().unwrap(), 3));

    assert!(is_path_id(content_iterator.next().unwrap(), 7));
    assert!(is_path_id(content_iterator.next().unwrap(), 8));
    assert!(is_path_id(content_iterator.next().unwrap(), 9));

    assert!(content_iterator.next().is_none());
}
