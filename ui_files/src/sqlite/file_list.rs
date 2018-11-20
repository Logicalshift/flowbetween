use super::file_error::*;

use rusqlite::*;

use std::result;
use std::path::{Path, PathBuf};

/// The definition file for the latest version of the database
const DEFINITION: &[u8]         = include_bytes!["../../sql/file_list_v2.sqlite"];

/// Performs the v1 to v2 upgrade steps
const UPGRADE_V1_TO_V2: &[u8]   = include_bytes!["../../sql/file_list_v1_to_v2.sqlite"];

/// The maximum supported version number
const MAX_VERSION: i64      = 2;

///
/// Manages a file list database
/// 
pub struct FileList {
    connection: Connection
}

impl FileList {
    ///
    /// Creates a new file list from a Sqlite connection
    /// 
    pub fn new(database_connection: Connection) -> result::Result<FileList, FileListError> {
        let connection_version = Self::version_number(&database_connection);

        if let Some(connection_version) = connection_version {
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
            None                => { self.initialize().map_err(|sqlerr| FileListError::SqlError(sqlerr))?; },
            Some(1)             => { self.upgrade_v1_to_v2()?; self.upgrade_to_latest()?; }
            Some(MAX_VERSION)   => { }

            _                   => { return result::Result::Err(FileListError::CannotUpgradeVersion); }
        }

        Ok(())
    }

    ///
    /// Upgrades from version 1 of the database to version 2
    ///
    fn upgrade_v1_to_v2(&mut self) -> result::Result<(), FileListError> {
        // Perform the upgrade in a transaction
        let transaction = self.connection.transaction().map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

        {
            // Create the version table marking this as a v2 database
            transaction.execute_batch(&String::from_utf8_lossy(UPGRADE_V1_TO_V2)).map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

            // Assign IDs to everything
            let mut existing_files  = transaction.prepare("SELECT RelativePath FROM Flo_Files").map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
            let existing_files      = existing_files.query_map::<String, _>(&[], |file| file.get(0)).map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

            let mut file_ids        = vec![];
            let mut add_id          = transaction.prepare("INSERT INTO Flo_Entity_Ordering(NextEntity) VALUES (NULL)").map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
            let mut update_id       = transaction.prepare("UPDATE Flo_Files SET EntityId = ? WHERE RelativePath = ?").map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

            for relative_path in existing_files {
                let relative_path = relative_path.map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

                // Generate an ID
                let new_id = add_id.insert(&[]).map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
                file_ids.push(new_id);

                // Update this file
                update_id.execute(&[&new_id, &relative_path]).map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
            }

            // Entity ID should now be unique
            transaction.execute_batch("CREATE UNIQUE INDEX Idx_Files_Entity ON Flo_Files (EntityId);").map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

            // Set the file ordering
            let mut set_next_entity = transaction.prepare("UPDATE Flo_Entity_Ordering SET NextEntity = ? WHERE EntityId = ?").map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
            for next_id_idx in 1..file_ids.len() {
                set_next_entity.execute(&[&file_ids[next_id_idx], &file_ids[next_id_idx-1]]).map_err(|sqlerr| FileListError::SqlError(sqlerr))?;
            }
        }

        // Commit the transaction
        transaction.commit().map_err(|sqlerr| FileListError::SqlError(sqlerr))?;

        // Upgrade was successful
        Ok(())
    }

    ///
    /// Returns the version number of this file list
    ///
    fn version_number(connection: &Connection) -> Option<i64> {
        // Try to fetch from the version number table
        let version_number  = connection.prepare("SELECT MAX(VersionNumber) FROM Flo_Files_Version");
        let version_number  = version_number.and_then(|mut version_number| version_number.query_row(&[], |row| row.get(0)));

        if let Ok(version_number) = version_number {
            // Database has a version number in it
            version_number
        } else {
            // V1 had no version number
            let all_files = connection.prepare("SELECT COUNT(*) FROM Flo_Files");

            if all_files.and_then(|mut all_files| all_files.query_row::<i64, _>(&[], |row| row.get(0))).is_ok() {
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
    fn add_entity(&self) -> i64 {
        let mut add_entity = self.connection.prepare("INSERT INTO Flo_Entity_Ordering(NextEntity) VALUES (NULL)").unwrap();
        add_entity.insert(&[]).unwrap()
    }

    ///
    /// Adds a path to the database
    /// 
    pub fn add_path(&self, path: &Path) {
        let path_string = Self::string_for_path(path);

        // Create an entity for this new file
        let entity_id = self.add_entity();

        // Create the file
        self.connection.execute("INSERT INTO Flo_Files (RelativePath, EntityId) VALUES (?, ?)", &[&path_string, &entity_id]).unwrap();
    }

    /*
    ///
    /// Deletes a path from the database
    /// 
    pub fn remove_path(&self, path: &Path) {
        let path_string = Self::string_for_path(path);
        self.connection.execute("DELETE FROM Flo_Files WHERE RelativePath = ?", &[&path_string]).unwrap();
    }
    */

    ///
    /// Lists the paths in the database
    /// 
    pub fn list_paths(&self) -> Vec<PathBuf> {
        let mut select_paths    = self.connection.prepare("SELECT RelativePath FROM Flo_Files").unwrap();
        let paths               = select_paths
            .query_map(&[], |row| {
                let path_string = row.get::<_, String>(0);
                let mut path    = PathBuf::new();
                path.push(path_string);

                path
            }).unwrap()
            .filter_map(|row| row.ok())
            .collect();
        
        paths
    }

    ///
    /// Updates the display name for a path
    /// 
    pub fn set_display_name_for_path(&self, path: &Path, display_name: &str) {
        let path_string = Self::string_for_path(path);

        self.connection.execute("UPDATE Flo_Files SET DisplayName = ? WHERE RelativePath = ?", &[&display_name, &path_string]).unwrap();
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
        let db          = Connection::open_in_memory().unwrap();
        let file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path());
    }

    #[test]
    pub fn add_many_paths() {
        let db          = Connection::open_in_memory().unwrap();
        let file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test1").as_path());
        file_list.add_path(&PathBuf::from("test2").as_path());
        file_list.add_path(&PathBuf::from("test3").as_path());
        file_list.add_path(&PathBuf::from("test4").as_path());
    }

    #[test]
    pub fn set_display_name() {
        let db          = Connection::open_in_memory().unwrap();
        let file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path());
        file_list.set_display_name_for_path(&PathBuf::from("test").as_path(), "TestDisplayName");

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
    fn get_version_latest() {
        let db          = Connection::open_in_memory().unwrap();
        let file_list   = FileList::new(db).unwrap();

        assert!(FileList::version_number(&file_list.connection) == Some(MAX_VERSION));
    }

    #[test]
    fn add_path_v1() {
        let db          = v1_database();
        let file_list   = FileList::new(db).unwrap();

        file_list.add_path(&PathBuf::from("test").as_path());
    }
}
