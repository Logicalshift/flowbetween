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
