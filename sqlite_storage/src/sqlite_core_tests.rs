use flo_animation::storage::*;

use rusqlite;
use super::sqlite_core::*;

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

    println!("{:?}", core.run_commands(vec![StorageCommand::ReadEditLogLength]));
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

    println!("{:?}", core.run_commands(vec![StorageCommand::ReadHighestUnusedElementId]));
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
