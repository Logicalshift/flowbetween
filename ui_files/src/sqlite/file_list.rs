use rusqlite::*;

use std::path::{Path, PathBuf};

const DEFINITION: &[u8]      = include_bytes!["../../sql/file_list.sqlite"];

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
    pub fn new(database_connection: Connection) -> FileList {
        FileList {
            connection: database_connection
        }
    }

    ///
    /// Initializes this file list
    /// 
    pub fn initialize(&self) -> Result<()> {
        // Create the definition string
        let definition   = String::from_utf8_lossy(DEFINITION);

        // Execute against the database
        self.connection.execute_batch(&definition)?;

        Ok(())
    }

    ///
    /// Returns the database representation of a path
    /// 
    fn string_for_path(path: &Path) -> String {
        // We take the canonical form of the path
        let canonical_path = path.canonicalize().ok();

        // As a safety measure, we don't allow any directories so only the last path component is used
        let final_component = canonical_path
            .and_then(|path| path.components()
                .last()
                .map(|component| component.as_os_str().to_string_lossy())
                .map(|component| String::from(component)));

        final_component.unwrap_or_else(|| String::from(""))
    }

    ///
    /// Adds a path to the database
    /// 
    pub fn add_path(&self, path: &Path) {
        let path_string = Self::string_for_path(path);
        self.connection.execute("INSERT INTO Flo_Files (RelativePath) VALUES (?)", &[&path_string]).unwrap();
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

        self.connection.query_row("SELECT DisplayName WHERE RelativePath = ?", &[&path_string], |row| row.get(0)).ok()
    }
}
