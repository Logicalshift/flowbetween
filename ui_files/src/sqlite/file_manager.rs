use super::file_list::*;
use super::super::file_manager::*;

use dirs;
use uuid::*;
use desync::*;
use rusqlite::*;

use std::fs;
use std::path::{Path, PathBuf};

const FILES_DB: &str = "files.db";
const DATA_DIR: &str = "data";

struct SqliteFileManagerCore {
    /// The database containing the list of files
    file_list: FileList
}

///
/// A file manager that uses Sqlite 
/// 
pub struct SqliteFileManager {
    /// Where we store our files
    root_path: PathBuf,

    /// The core of this file manager
    core: Desync<SqliteFileManagerCore>
}

impl SqliteFileManager {
    ///
    /// Creates a new Sqlite file manager (in a sub-path of the main files directory)
    /// 
    pub fn new(application_path: &str, sub_path: &str) -> SqliteFileManager {
        // This will be the 'root' data directory for the user
        let mut data_dir = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .unwrap();

        // Append the path components
        data_dir.push(application_path);
        data_dir.push(sub_path);

        // Create the data directory if it does not exist
        fs::create_dir_all(data_dir.as_path()).unwrap();

        // Check for the file list database file
        let mut database_file = data_dir.clone();
        database_file.push(FILES_DB);

        // Connect to the Sqlite database
        let database_file_exists    = database_file.is_file();
        let database_connection     = Connection::open(database_file.as_path()).unwrap();
        let file_list               = FileList::new(database_connection);

        if !database_file_exists {
            file_list.initialize().unwrap();
        }

        // Put together the file manager
        SqliteFileManager {
            root_path:  data_dir,
            core:       Desync::new(SqliteFileManagerCore {
                file_list: file_list
            })
        }
    }
}

impl FileManager for SqliteFileManager {
    ///
    /// Returns a list of all the files that can be opened by this manager
    /// 
    fn get_all_files(&self) -> Vec<PathBuf> {
        // Retrieve from the file list and append the folder we're using
        self.core.sync(|core| core.file_list.list_paths())
            .into_iter()
            .map(|last_component| {
                let mut full_path = self.root_path.clone();
                full_path.push(DATA_DIR);
                full_path.push(last_component);
                full_path
            })
            .collect()
    }

    ///
    /// Returns the display name for a particular path
    /// 
    fn display_name_for_path(&self, path: &Path) -> Option<String> {
        // TODO: use the path from get_all_files
        self.core.sync(|core| core.file_list.display_name_for_path(path))
    }

    ///
    /// Reserves a path for a new file (this path is valid and won't be re-used by future calls but
    /// no files will exist here yet)
    /// 
    fn create_new_path(&self) -> PathBuf {
        // Generate a filename
        let filename        = Uuid::new_v4().simple().to_string();
        let mut full_path   = self.root_path.clone();

        full_path.push(DATA_DIR);
        full_path.push(&filename);

        // Add to the database
        let mut filename_buf = PathBuf::new();
        filename_buf.push(filename);
        self.core.async(move |core| core.file_list.add_path(filename_buf.as_path()));

        // Result is the full path
        full_path
    }

    ///
    /// Updates or creates the display name associated with a particular path (which must be
    /// returned via get_all_files: setting the name for a non-existent path will just
    /// result)
    ///
    fn set_display_name_for_path(&self, path: &Path, display_name: String) {
        let path = PathBuf::from(path);

        self.core.async(move |core| core.file_list.set_display_name_for_path(path.as_path(), &display_name))
    }
}
