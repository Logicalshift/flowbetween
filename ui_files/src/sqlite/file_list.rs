use super::file_error::*;

use flo_logging::*;

use rusqlite::*;

use std::result;
use std::path::{Path, PathBuf};

/// The definition file for the latest version of the database
const DEFINITION: &[u8]         = include_bytes!["../../sql/file_list_v2.sqlite"];

/// The maximum supported version number
const MAX_VERSION: i64      = 2;

/// The ID of the root entity (where the standard file directory is located)
const ROOT_ENTITY: i64      = -1;

///
/// Manages a file list database
///
pub struct FileList {
    log: LogPublisher,
    connection: Connection
}

impl FileList {
    ///
    /// Creates a new file list from a Sqlite connection
    ///
    pub fn new(database_connection: Connection) -> result::Result<FileList, FileListError> {
        let log                 = LogPublisher::new(module_path!());
        let connection_version  = Self::version_number(&database_connection);

        if let Some(connection_version) = connection_version {
            log.log((Level::Info, format!("File list database version {}", connection_version)));

            // Check the version is in the valid ranges
            if connection_version <= 0 {
                // Minimum version is 1
                return result::Result::Err(FileListError::BadVersionNumber("Files database has an invalid version number (less than zero)".to_string()));
            } else if connection_version > MAX_VERSION {
                return result::Result::Err(FileListError::BadVersionNumber("Files database appears to be from a newer version of this tool".to_string()));
            }
        }

        // Create the result
        let mut result = FileList {
            log:        log,
            connection: database_connection
        };

        // Upgrade the contents if necessary
        result.upgrade_to_latest()?;

        Ok(result)
    }

    ///
    /// Initializes this file list
    ///
    fn initialize(&self) -> Result<()> {
        self.log.log((Level::Info, "Initializing file list database"));

        // Create the definition string
        let definition   = String::from_utf8_lossy(DEFINITION);

        // Execute against the database
        self.connection.execute_batch(&definition)?;

        Ok(())
    }

    ///
    /// Attempts to upgrade the database to the latest version
    ///
    fn upgrade_to_latest(&mut self) -> result::Result<(), FileListError> {
        // Get the current version
        let connection_version = Self::version_number(&self.connection);

        match connection_version {
            None                => { self.initialize()?; },
            Some(1)             => { Self::upgrade_v1_to_v2(&self.log, &mut self.connection)?; self.upgrade_to_latest()?; }
            Some(MAX_VERSION)   => { }

            _                   => { return result::Result::Err(FileListError::CannotUpgradeVersion); }
        }

        Ok(())
    }

    ///
    /// Returns the version number of this file list
    ///
    fn version_number(connection: &Connection) -> Option<i64> {
        // Try to fetch from the version number table
        let version_number  = connection.prepare("SELECT MAX(VersionNumber) FROM Flo_Files_Version");
        let version_number  = version_number.and_then(|mut version_number| version_number.query_row(NO_PARAMS, |row| row.get(0)));

        if let Ok(version_number) = version_number {
            // Database has a version number in it
            version_number
        } else {
            // V1 had no version number
            let all_files = connection.prepare("SELECT COUNT(*) FROM Flo_Files");

            if all_files.and_then(|mut all_files| all_files.query_row::<i64, _, _>(NO_PARAMS, |row| row.get(0))).is_ok() {
                // V1
                Some(1)
            } else {
                // Not a flo_files database
                None
            }
        }
    }

    ///
    /// Returns the database representation of a path
    ///
    fn string_for_path(path: &Path) -> String {
        // As a safety measure, we don't allow any directories so only the last path component is used
        let final_component = path.components()
            .last()
            .map(|component| component.as_os_str().to_string_lossy())
            .map(|component| String::from(component));

        final_component.unwrap_or_else(|| String::from(""))
    }

    ///
    /// Creates a new entity in the database
    ///
    fn add_entity(transaction: &Transaction) -> result::Result<i64, FileListError> {
        let mut add_entity = transaction.prepare("INSERT INTO Flo_Entity_Ordering(NextEntity) VALUES (?)")?;
        Ok(add_entity.insert(&[&ROOT_ENTITY])?)
    }

    ///
    /// Makes the specified entity the first entity in the database for the specified parent entity
    ///
    fn make_first_entity(transaction: &Transaction, entity_id: i64, parent_entity_id: i64) -> result::Result<(), FileListError> {
        // 'Orphan' entities are entities with no previous entity
        let mut orphan_entities = transaction.prepare("SELECT EntityId FROM Flo_Entity_Ordering WHERE ParentEntityId = ? AND EntityId != ? AND EntityId NOT IN (SELECT NextEntity FROM Flo_Entity_Ordering)")?;
        let orphan_entities     = orphan_entities.query_map(&[&parent_entity_id, &entity_id], |row| row.get(0))?;
        let orphan_entities     = orphan_entities.filter_map(|item| item.ok()).collect::<Vec<i64>>();

        if orphan_entities.len() == 0 {
            // Is the first entity in the list
            Ok(())
        } else if orphan_entities.len() == 1 {
            // Set the next entity of the current entity to the first 'orphan' entity
            let mut set_next_entity = transaction.prepare("UPDATE Flo_Entity_Ordering SET NextEntity = ? WHERE EntityId = ?")?;
            set_next_entity.execute(&[&orphan_entities[0], &entity_id])?;

            Ok(())
        } else {
            // There's more than one start point for the list. Reduce to a single 'orphan' entity
            let mut set_next_entity = transaction.prepare("UPDATE Flo_Entity_Ordering SET NextEntity = ? WHERE EntityId = ?")?;
            set_next_entity.execute(&[&orphan_entities[0], &entity_id])?;

            for next_idx in 1..orphan_entities.len() {
                set_next_entity.execute(&[&orphan_entities[next_idx], &orphan_entities[next_idx-1]])?;
            }

            Ok(())
        }
    }

    ///
    /// Adds a path to the database
    ///
    pub fn add_path(&mut self, path: &Path) -> result::Result<(), FileListError> {
        let transaction = self.connection.transaction()?;

        let path_string = Self::string_for_path(path);

        // Create an entity for this new file
        let entity_id = Self::add_entity(&transaction)?;

        // Create the file
        transaction.execute::<&[&dyn ToSql]>("INSERT INTO Flo_Files (RelativePath, EntityId) VALUES (?, ?)", &[&path_string, &entity_id])?;

        // Make the file the first entity in the database
        Self::make_first_entity(&transaction, entity_id, ROOT_ENTITY)?;

        // Finish the transaction
        transaction.commit()?;

        Ok(())
    }

    ///
    /// Deletes a path from the database
    ///
    pub fn remove_path(&mut self, path: &Path) -> result::Result<(), FileListError> {
        let transaction = self.connection.transaction()?;

        // Unlink the entity
        let entity_id = Self::entity_id_for_path(&transaction, path)?;
        Self::unlink_entity(&transaction, entity_id)?;

        // Remove from the file list
        let path_string = Self::string_for_path(path);
        transaction.execute("DELETE FROM Flo_Files WHERE RelativePath = ?", &[&path_string]).unwrap();

        // Remove from the entity list
        transaction.execute("DELETE FROM Flo_Entity_Ordering WHERE EntityId = ?", &[&entity_id]).unwrap();

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Lists the paths in the database
    ///
    pub fn list_paths(&self) -> result::Result<Vec<PathBuf>, FileListError> {
        let mut select_paths    = self.connection.prepare("
            WITH RECURSIVE RootEntities AS (
                SELECT 0 AS idx, EntityId, NextEntity
                    FROM    Flo_Entity_Ordering
                    WHERE   ParentEntityId = ? AND EntityId NOT IN (SELECT NextEntity FROM Flo_Entity_Ordering)
                UNION
                SELECT RootEntities.idx+1, Flo_Entity_Ordering.EntityId, Flo_Entity_Ordering.NextEntity
                    FROM    Flo_Entity_Ordering, RootEntities
                    WHERE   Flo_Entity_Ordering.EntityId = RootEntities.NextEntity
                    AND     Flo_Entity_Ordering.EntityId != -1
            )
            SELECT RelativePath FROM Flo_Files
            INNER JOIN RootEntities ON Flo_Files.EntityId = RootEntities.EntityId
            ORDER BY RootEntities.idx
        ")?;
        let paths               = select_paths
            .query_map(&[&ROOT_ENTITY], |row| {
                let path_string = row.get::<_, String>(0)?;
                let mut path    = PathBuf::new();
                path.push(path_string);

                Ok(path)
            })?
            .filter_map(|row| row.ok())
            .collect();

        Ok(paths)
    }

    ///
    /// Retrieves the entity for a particular path
    ///
    fn entity_id_for_path(transaction: &Transaction, path: &Path) -> result::Result<i64, FileListError> {
        let entity_id = transaction.query_row("SELECT EntityId FROM Flo_Files WHERE RelativePath = ?", &[&Self::string_for_path(path)], |row| row.get(0))?;
        Ok(entity_id)
    }

    ///
    /// Retrieve the following entity ID for a given entity
    ///
    fn get_next_entity_id(transaction: &Transaction, entity_id: i64) -> result::Result<i64, FileListError> {
        let next_entity_id = transaction.query_row("SELECT NextEntity FROM Flo_Entity_Ordering WHERE EntityId = ?", &[&entity_id], |row| row.get(0))?;

        Ok(next_entity_id)
    }

    ///
    /// Retrieve the previous entity ID for a given entity (ROOT_ENTITY if it has none)
    ///
    fn get_previous_entity_id(transaction: &Transaction, entity_id: i64) -> result::Result<i64, FileListError> {
        let mut previous_entity_id  = transaction.prepare("SELECT EntityId FROM Flo_Entity_Ordering WHERE NextEntity = ?")?;
        let mut previous_entity_id  = previous_entity_id.query_map(&[&entity_id], |row| row.get(0))?;
        let previous_entity_id      = previous_entity_id.nth(0);

        Ok(previous_entity_id.unwrap_or(Ok(ROOT_ENTITY)).unwrap_or(ROOT_ENTITY))
    }

    ///
    /// Retrieve the first entity for a given parent entity (or ROOT_ENTITY if it has none)
    ///
    fn get_first_entity_id(transaction: &Transaction, parent_entity_id: i64) -> result::Result<i64, FileListError> {
        let mut first_entity_id = transaction.prepare("SELECT EntityId FROM Flo_Entity_Ordering WHERE ParentEntityId = ? AND EntityId NOT IN (SELECT NextEntity FROM Flo_Entity_Ordering)")?;
        let mut first_entity_id = first_entity_id.query_map(&[&parent_entity_id], |row| row.get(0))?;
        let first_entity_id     = first_entity_id.nth(0);

        Ok(first_entity_id.unwrap_or(Ok(ROOT_ENTITY)).unwrap_or(ROOT_ENTITY))
    }

    ///
    /// Sets the next entity value for a particular entity
    ///
    fn set_next_entity(transaction: &Transaction, entity_id: i64, next_entity: i64) -> result::Result<(), FileListError> {
        transaction.execute("UPDATE Flo_Entity_Ordering SET NextEntity = ? WHERE EntityId = ?", &[&next_entity, &entity_id])?;
        Ok(())
    }

    ///
    /// Removes the links for a particular entity
    ///
    fn unlink_entity(transaction: &Transaction, entity_id: i64) -> result::Result<(), FileListError> {
        // Get the previous and next entities for the current entity
        let previous_entity = Self::get_previous_entity_id(&transaction, entity_id)?;
        let next_entity     = Self::get_next_entity_id(&transaction, entity_id)?;

        // Unset the next entity for the current entity
        Self::set_next_entity(transaction, entity_id, ROOT_ENTITY)?;

        // If there's a previous entity, set its next entity to old value of our entity
        if previous_entity != ROOT_ENTITY {
            Self::set_next_entity(transaction, previous_entity, next_entity)?;
        }

        Ok(())
    }

    ///
    /// Inserts an entity after the specified entity
    ///
    fn insert_after(transaction: &Transaction, entity_id: i64, after: i64) -> result::Result<(), FileListError> {
        // Nothing to do if trying to move an item after itself
        if after == entity_id {
            return Ok(());
        }

        // Move to the beginning if 'after' is the root entity
        if after == ROOT_ENTITY {
            // Get the first entity in the list
            let first_entity_id = Self::get_first_entity_id(transaction, ROOT_ENTITY)?;

            if first_entity_id != entity_id {
                // Remove the entity from the list
                Self::unlink_entity(transaction, entity_id)?;

                // Next entity becomes the old first entity
                Self::set_next_entity(transaction, entity_id, first_entity_id)?;
            }
        } else {
            // Remove the entity from the list
            Self::unlink_entity(transaction, entity_id)?;

            // Make the next entity be the same as the one currently after
            let next_after = Self::get_next_entity_id(transaction, after)?;
            Self::set_next_entity(transaction, entity_id, next_after)?;

            // Insert the original entity after the specified one
            Self::set_next_entity(transaction, after, entity_id)?;
        }

        Ok(())
    }

    ///
    /// Re-orders a path so that it appears after a particular path (or None to appear at the beginning)
    ///
    pub fn order_path_after(&mut self, path_to_move: &Path, path_to_move_after: Option<&Path>) -> result::Result<(), FileListError> {
        let transaction             = self.connection.transaction()?;

        // Get the entity IDs for the paths
        let path_to_move_id         = Self::entity_id_for_path(&transaction, path_to_move)?;
        let path_to_move_after_id   = if let Some(path_to_move_after) = path_to_move_after {
            Self::entity_id_for_path(&transaction, path_to_move_after)?
        } else {
            ROOT_ENTITY
        };

        // Move the entity ID order
        Self::insert_after(&transaction, path_to_move_id, path_to_move_after_id)?;

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Updates the display name for a path
    ///
    pub fn set_display_name_for_path(&self, path: &Path, display_name: &str) -> result::Result<(), FileListError> {
        let path_string = Self::string_for_path(path);

        self.connection.execute::<&[&dyn ToSql]>("UPDATE Flo_Files SET DisplayName = ? WHERE RelativePath = ?", &[&display_name, &path_string])?;

        Ok(())
    }

    ///
    /// Retrieves the display name for a particular path
    ///
    pub fn display_name_for_path(&self, path: &Path) -> Option<String> {
        let path_string = Self::string_for_path(path);

        self.connection.query_row("SELECT DisplayName FROM Flo_Files WHERE RelativePath = ?", &[&path_string], |row| row.get(0)).ok().and_then(|name| name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn v1_database() -> Connection {
        let db          = Connection::open_in_memory().unwrap();

        // Create the definition string
        let definition   = String::from_utf8_lossy(include_bytes!["../../sql/file_list_v1.sqlite"]);

        // Execute against the database
        db.execute_batch(&definition).unwrap();

        db
    }

    #[test]
    pub fn initialize() {
        let db          = Connection::open_in_memory().unwrap();
        let _file_list  = FileList::new(db).unwrap();
    }

    #[test]
    pub fn add_path() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path()).unwrap();
    }

    #[test]
    pub fn add_many_paths() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();
    }

    #[test]
    pub fn paths_list_in_reverse_order_by_default() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test4".to_string(),
            "test3".to_string(),
            "test2".to_string(),
            "test1".to_string()
        ]);
    }

    #[test]
    pub fn can_move_paths() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        file_list.order_path_after(&PathBuf::from("test3").as_path(), Some(&PathBuf::from("test2").as_path())).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test4".to_string(),
            "test2".to_string(),
            "test3".to_string(),
            "test1".to_string()
        ]);
    }

    #[test]
    pub fn can_move_path_to_start() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        file_list.order_path_after(&PathBuf::from("test3").as_path(), None).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test3".to_string(),
            "test4".to_string(),
            "test2".to_string(),
            "test1".to_string()
        ]);
    }

    #[test]
    pub fn can_move_path_from_start() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        file_list.order_path_after(&PathBuf::from("test4").as_path(), Some(&PathBuf::from("test1").as_path())).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test3".to_string(),
            "test2".to_string(),
            "test1".to_string(),
            "test4".to_string()
        ]);
    }

    #[test]
    pub fn can_move_path_at_start_back_to_start() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        file_list.order_path_after(&PathBuf::from("test4").as_path(), None).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test4".to_string(),
            "test3".to_string(),
            "test2".to_string(),
            "test1".to_string(),
        ]);
    }

    #[test]
    pub fn can_move_path_over_self() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test2").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test3").as_path()).unwrap();
        file_list.add_path(&PathBuf::from("test4").as_path()).unwrap();

        file_list.order_path_after(&PathBuf::from("test4").as_path(), Some(&PathBuf::from("test4").as_path())).unwrap();

        let paths = file_list.list_paths().unwrap();
        let paths = paths.into_iter().map(|path_buf| path_buf.to_str().unwrap().to_string()).collect::<Vec<_>>();

        assert!(paths == vec![
            "test4".to_string(),
            "test3".to_string(),
            "test2".to_string(),
            "test1".to_string(),
        ]);
    }

    #[test]
    pub fn set_display_name() {
        let db              = Connection::open_in_memory().unwrap();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path()).unwrap();
        file_list.set_display_name_for_path(&PathBuf::from("test").as_path(), "TestDisplayName").unwrap();

        assert!(file_list.display_name_for_path(&PathBuf::from("test").as_path()) == Some("TestDisplayName".to_string()));
    }

    #[test]
    fn get_version_uninitialized() {
        let db          = Connection::open_in_memory().unwrap();

        assert!(FileList::version_number(&db) == None);
    }

    #[test]
    fn get_version_v1() {
        let db          = v1_database();

        assert!(FileList::version_number(&db) == Some(1));
    }

    #[test]
    fn upgrade_v1() {
        let db          = v1_database();
        let file_list   = FileList::new(db).unwrap();

        assert!(FileList::version_number(&file_list.connection) == Some(MAX_VERSION));
    }

    #[test]
    fn get_version_latest() {
        let db          = Connection::open_in_memory().unwrap();
        let file_list   = FileList::new(db).unwrap();

        assert!(FileList::version_number(&file_list.connection) == Some(MAX_VERSION));
    }

    #[test]
    fn add_path_v1() {
        let db              = v1_database();
        let mut file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path()).unwrap();
    }

}
