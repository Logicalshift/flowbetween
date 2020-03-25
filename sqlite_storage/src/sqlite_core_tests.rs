use flo_animation::storage::*;

use rusqlite;
use super::sqlite_core::*;

use std::i64;
use std::time::{Duration};

#[test]
fn initialize_database() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    let result      = core.initialize();

    if result.is_err() { println!("{:?}", result.as_ref().err()) }

    assert!(result.is_ok());
}

#[test]
fn read_no_properties() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    let no_props    = core.run_commands(vec![StorageCommand::ReadAnimationProperties]);
    assert!(no_props == vec![StorageResponse::NotFound]);
}

#[test]
fn read_and_write_properties() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    let no_props    = core.run_commands(vec![StorageCommand::WriteAnimationProperties("Test".to_string())]);
    assert!(no_props == vec![StorageResponse::Updated]);

    let props       = core.run_commands(vec![StorageCommand::ReadAnimationProperties]);
    assert!(props == vec![StorageResponse::AnimationProperties("Test".to_string())]);
}

#[test]
fn edit_log_is_initially_empty() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![StorageCommand::ReadEditLogLength]) == vec![StorageResponse::NumberOfEdits(0)]);
}

#[test]
fn write_two_edits() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::WriteEdit("Test1".to_string()), 
            StorageCommand::WriteEdit("Test2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadEditLogLength]) == vec![StorageResponse::NumberOfEdits(2)]);
}

#[test]
fn read_all_edits() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::WriteEdit("Test1".to_string()), 
            StorageCommand::WriteEdit("Test2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadEdits(0..2)]) == vec![StorageResponse::Edit(0, "Test1".to_string()), StorageResponse::Edit(1, "Test2".to_string())]);
}

#[test]
fn highest_unused_element_id_is_0_with_no_elements() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![StorageCommand::ReadHighestUnusedElementId]) == vec![StorageResponse::HighestUnusedElementId(0)]);
}

#[test]
fn read_missing_element() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![StorageCommand::ReadElement(1)]) == vec![StorageResponse::NotFound]);
}

#[test]
fn write_and_read_elements() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::WriteElement(1, "Test1".to_string()), 
            StorageCommand::WriteElement(3, "Test2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElement(1), StorageCommand::ReadElement(3)]) == 
        vec![StorageResponse::Element(1, "Test1".to_string()), StorageResponse::Element(3, "Test2".to_string())]);
}

#[test]
fn write_and_delete_elements() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::WriteElement(1, "Test1".to_string()), 
            StorageCommand::WriteElement(3, "Test2".to_string()),
            StorageCommand::DeleteElement(3)
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElement(1), StorageCommand::ReadElement(3)]) == 
        vec![StorageResponse::Element(1, "Test1".to_string()), StorageResponse::NotFound]);
}

#[test]
fn read_no_layers() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![StorageCommand::ReadLayers]) == 
        vec![]);
}

#[test]
fn add_and_read_layers() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddLayer(3, "Test2".to_string()),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayers]) == 
        vec![StorageResponse::LayerProperties(1, "Test1".to_string()), StorageResponse::LayerProperties(3, "Test2".to_string())]);
}

#[test]
fn write_layer_properties() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddLayer(3, "Test2".to_string()),
            StorageCommand::WriteLayerProperties(3, "Test3".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayers]) == 
        vec![StorageResponse::LayerProperties(1, "Test1".to_string()), StorageResponse::LayerProperties(3, "Test3".to_string())]);
}

#[test]
fn add_and_delete_layers() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddLayer(3, "Test2".to_string()),
            StorageCommand::DeleteLayer(3)
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayers]) == 
        vec![StorageResponse::LayerProperties(1, "Test1".to_string())]);
}

#[test]
fn read_layer_properties() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddLayer(3, "Test2".to_string()),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerProperties(3)]) == 
        vec![StorageResponse::LayerProperties(3, "Test2".to_string())]);
}

#[test]
fn read_missing_layer_properties() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![StorageCommand::ReadLayerProperties(3)]) == 
        vec![StorageResponse::NotFound]);
}

#[test]
fn add_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(420)..Duration::from_millis(420))])
        == vec![StorageResponse::KeyFrame(Duration::from_millis(420), Duration::from_micros(i64::MAX as u64))]);
}

#[test]
fn read_many_keyframes() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(600)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(700))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(500)..Duration::from_millis(500))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600))
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(502)..Duration::from_millis(502))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600))
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(0)..Duration::from_millis(599))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(420), Duration::from_millis(500)),
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600))
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(500)..Duration::from_millis(700))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600)),
            StorageResponse::KeyFrame(Duration::from_millis(600), Duration::from_millis(700))
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(500)..Duration::from_millis(701))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600)),
            StorageResponse::KeyFrame(Duration::from_millis(600), Duration::from_millis(700)),
            StorageResponse::KeyFrame(Duration::from_millis(700), Duration::from_micros(i64::MAX as u64))
        ]);
}

#[test]
fn read_missing_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(600)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(700))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(0)..Duration::from_millis(1))]) ==
        vec![
            StorageResponse::NotInAFrame(Duration::from_millis(420))
        ]);
}

#[test]
fn read_with_no_keyframes() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(700)..Duration::from_millis(701))]) ==
        vec![
        ]);
}

#[test]
fn read_past_last_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(600)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(700))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(701)..Duration::from_millis(702))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(700), Duration::from_micros(i64::MAX as u64))
        ]);
}

#[test]
fn delete_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(600)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(700))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![
            StorageCommand::DeleteKeyFrame(1, Duration::from_millis(500))
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(0)..Duration::from_millis(700))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(420), Duration::from_millis(600)),
            StorageResponse::KeyFrame(Duration::from_millis(600), Duration::from_millis(700))
        ]);
}

#[test]
fn delete_nonexistent_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(600)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(700))
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![
            StorageCommand::DeleteKeyFrame(1, Duration::from_millis(550))
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadKeyFrames(1, Duration::from_millis(0)..Duration::from_millis(700))]) ==
        vec![
            StorageResponse::KeyFrame(Duration::from_millis(420), Duration::from_millis(500)),
            StorageResponse::KeyFrame(Duration::from_millis(500), Duration::from_millis(600)),
            StorageResponse::KeyFrame(Duration::from_millis(600), Duration::from_millis(700))
        ]);
}

#[test]
fn attach_element_to_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 

            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),

            StorageCommand::WriteElement(1, "Test1".to_string()),
            StorageCommand::WriteElement(2, "Test2".to_string()),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(420))]) ==
        vec![
            StorageResponse::Element(1, "Test1".to_string()),
            StorageResponse::Element(2, "Test2".to_string()),
        ]);
}

#[test]
fn attach_element_to_several_keyframes() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 

            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),

            StorageCommand::WriteElement(1, "Test1".to_string()),
            StorageCommand::WriteElement(2, "Test2".to_string()),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),

            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(500)),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(420))]) ==
        vec![
            StorageResponse::Element(1, "Test1".to_string()),
            StorageResponse::Element(2, "Test2".to_string()),
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(500))]) ==
        vec![
            StorageResponse::Element(2, "Test2".to_string()),
        ]);
}

#[test]
fn double_attach_element_to_keyframe() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 

            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),

            StorageCommand::WriteElement(1, "Test1".to_string()),
            StorageCommand::WriteElement(2, "Test2".to_string()),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(420))]) ==
        vec![
            StorageResponse::Element(1, "Test1".to_string()),
            StorageResponse::Element(2, "Test2".to_string()),
        ]);
}

#[test]
fn detach_element() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 

            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),

            StorageCommand::WriteElement(1, "Test1".to_string()),
            StorageCommand::WriteElement(2, "Test2".to_string()),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),

            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(500)),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![
            StorageCommand::DetachElementFromLayer(2), 
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(420))]) ==
        vec![
            StorageResponse::Element(1, "Test1".to_string()),
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementsForKeyFrame(1, Duration::from_millis(500))]) ==
        vec![
        ]);
}

#[test]
fn read_element_attachments() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 

            StorageCommand::AddKeyFrame(1, Duration::from_millis(420)),
            StorageCommand::AddKeyFrame(1, Duration::from_millis(500)),

            StorageCommand::WriteElement(1, "Test1".to_string()),
            StorageCommand::WriteElement(2, "Test2".to_string()),

            StorageCommand::AttachElementToLayer(1, 1, Duration::from_millis(420)),
            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(420)),

            StorageCommand::AttachElementToLayer(1, 2, Duration::from_millis(500)),
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementAttachments(1)]) ==
        vec![
            StorageResponse::ElementAttachments(1, vec![(1, Duration::from_millis(420))]),
        ]);

    assert!(core.run_commands(vec![StorageCommand::ReadElementAttachments(2)]) ==
        vec![
            StorageResponse::ElementAttachments(2, vec![(1, Duration::from_millis(420)), (1, Duration::from_millis(500))]),
        ]);
}

#[test]
fn read_layer_cache() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::WriteLayerCache(1, Duration::from_millis(420), "Type".to_string(), "Cache1".to_string()),
            StorageCommand::WriteLayerCache(1, Duration::from_millis(500), "Type".to_string(), "Cache2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(420), "Type".to_string())]) ==
        vec![StorageResponse::LayerCache("Cache1".to_string())]);
    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(500), "Type".to_string())]) ==
        vec![StorageResponse::LayerCache("Cache2".to_string())]);
}

#[test]
fn overwrite_layer_cache() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::WriteLayerCache(1, Duration::from_millis(420), "Type".to_string(), "Cache1".to_string()),
            StorageCommand::WriteLayerCache(1, Duration::from_millis(500), "Type".to_string(), "Cache2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![
            StorageCommand::WriteLayerCache(1, Duration::from_millis(420), "Type".to_string(), "Cache1Updated".to_string()),
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(420), "Type".to_string())]) ==
        vec![StorageResponse::LayerCache("Cache1Updated".to_string())]);
    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(500), "Type".to_string())]) ==
        vec![StorageResponse::LayerCache("Cache2".to_string())]);
}

#[test]
fn read_missing_layer_cache() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::WriteLayerCache(1, Duration::from_millis(420), "Type".to_string(), "Cache1".to_string()),
            StorageCommand::WriteLayerCache(1, Duration::from_millis(500), "Type".to_string(), "Cache2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(600), "Type".to_string())]) ==
        vec![StorageResponse::NotFound]);
}

#[test]
fn read_empty_layer_cache() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string())
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(600), "Type".to_string())]) ==
        vec![StorageResponse::NotFound]);
}

#[test]
fn delete_from_layer_cache() {
    let mut core    = SqliteCore::new(rusqlite::Connection::open_in_memory().unwrap());
    core.initialize().unwrap();

    assert!(core.run_commands(vec![
            StorageCommand::AddLayer(1, "Test1".to_string()), 
            StorageCommand::WriteLayerCache(1, Duration::from_millis(420), "Type".to_string(), "Cache1".to_string()),
            StorageCommand::WriteLayerCache(1, Duration::from_millis(500), "Type".to_string(), "Cache2".to_string())
        ]) == vec![StorageResponse::Updated, StorageResponse::Updated, StorageResponse::Updated]);

    assert!(core.run_commands(vec![
            StorageCommand::DeleteLayerCache(1, Duration::from_millis(420), "Type".to_string()),
        ]) == vec![StorageResponse::Updated]);

    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(420), "Type".to_string())]) ==
        vec![StorageResponse::NotFound]);
    assert!(core.run_commands(vec![StorageCommand::ReadLayerCache(1, Duration::from_millis(500), "Type".to_string())]) ==
        vec![StorageResponse::LayerCache("Cache2".to_string())]);
}
