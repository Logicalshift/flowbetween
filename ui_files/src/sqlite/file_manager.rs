use super::file_list::*;
use super::super::file_update::*;
use super::super::file_manager::*;

use flo_stream::*;
use flo_logging::*;

use dirs;
use uuid::*;
use ::desync::*;
use rusqlite::*;
use futures::*;
use futures::stream::{BoxStream};

use std::fs;
use std::sync::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

const FILES_DB: &str = "files.db";
const DATA_DIR: &str = "data";

lazy_static! {
    // Exising file manager cores for particular application paths (and ensures only one can be being created at once)
    static ref FILE_CORES: Desync<HashMap<(String, String), Arc<Desync<SqliteFileManagerCore>>>> = Desync::new(HashMap::new());
}

struct SqliteFileManagerCore {
    /// The log for this file manager
    log: LogPublisher,

    /// Where we store our files
    root_path: PathBuf,

    /// The database containing the list of files
    file_list: FileList,

    /// The senders for updates to this file manager
    updates: Publisher<FileUpdate>
}

///
/// A file manager that uses Sqlite
///
pub struct SqliteFileManager {
    /// Where we store our files
    root_path: PathBuf,

    /// The core of this file manager
    core: Arc<Desync<SqliteFileManagerCore>>
}

impl SqliteFileManagerCore {
    ///
    /// Sends an update to everything that's listening for them
    ///
    pub fn send_update(&mut self, update: FileUpdate) -> impl Future<Output=()> {
        // Send to the update publisher
        let update = self.updates.publish(update);
        update
    }
}

impl SqliteFileManager {
    ///
    /// Creates a new file manager core
    ///
    fn new_core(application_path: &str, sub_path: &str) -> Arc<Desync<SqliteFileManagerCore>> {
        let log         = LogPublisher::new(module_path!());

        // This will be the 'root' data directory for the user
        let mut root_path = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .unwrap();

        // Append the path components
        root_path.push(application_path);
        root_path.push(sub_path);

        // Create the data directory if it does not exist
        fs::create_dir_all(root_path.as_path()).unwrap();

        // Create the subdirectories too
        let mut data_dir = root_path.clone();
        data_dir.push(DATA_DIR);
        fs::create_dir_all(data_dir.as_path()).unwrap();

        log.log((Level::Info, format!("Using data directory at `{}`", data_dir.to_str().unwrap_or("<Missing path>"))));

        // Check for the file list database file
        let mut database_file = root_path.clone();
        database_file.push(FILES_DB);

        // Connect to the Sqlite database
        let database_connection     = Connection::open(database_file.as_path()).unwrap();
        let file_list               = FileList::new(database_connection).unwrap();

        // Create the update publisher
        let update_publisher = Publisher::new(100);

        Arc::new(Desync::new(SqliteFileManagerCore {
            file_list:  file_list,
            root_path:  root_path,
            updates:    update_publisher,
            log:        log
        }))
    }

    ///
    /// Creates a new Sqlite file manager (in a sub-path of the main files directory)
    ///
    /// Separate sub-paths can be used to allow for multi-user scenarios: in single-user
    /// scenarios we usually set this to `"default"`.
    ///
    pub fn new(application_path: &str, sub_path: &str) -> SqliteFileManager {
        // Create the core, or use an existing one if there is one
        let core = FILE_CORES.sync(|file_cores|
            file_cores.entry((String::from(application_path), String::from(sub_path)))
            .or_insert_with(|| Self::new_core(application_path, sub_path))
            .clone());

        // Fetch information from it
        let root_path   = core.sync(|core| core.root_path.clone());

        // Put together the file manager
        SqliteFileManager {
            root_path:  root_path,
            core:       core
        }
    }

    ///
    /// Retrieves the log for this file manager
    ///
    pub fn log(&self) -> LogPublisher {
        self.core.sync(|core| core.log.clone())
    }

    ///
    /// Finds the path to request from the file list for a particular file path
    ///
    fn file_list_path(&self, path: &Path) -> Option<PathBuf> {
        // Construct a path representing where we store our data
        let mut data_path = self.root_path.clone();
        data_path.push(DATA_DIR);

        if path.components().count() == 1 && path.is_relative() {

            // A single relative component is left intact
            path.components().last()
                .map(|component| component.as_os_str().to_string_lossy())
                .map(|component| {
                    let mut buf = PathBuf::new();
                    buf.push(component.to_string());
                    buf
                })

        } else if path.starts_with(data_path) {

            // If the path is in the data path, then use the last component
            // TODO: this isn't quite right if the path is in a subdirectory
            path.components().last()
                .map(|component| component.as_os_str().to_string_lossy())
                .map(|component| {
                    let mut buf = PathBuf::new();
                    buf.push(component.to_string());
                    buf
                })

        } else {
            None
        }
    }
}

impl FileManager for SqliteFileManager {
    ///
    /// Returns a list of all the files that can be opened by this manager
    ///
    fn get_all_files(&self) -> Vec<PathBuf> {
        // Retrieve from the file list and append the folder we're using
        self.core.sync(|core| core.file_list.list_paths().unwrap())
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
        let path = self.file_list_path(path);

        if let Some(path) = path {
            self.core.sync(|core| core.file_list.display_name_for_path(path.as_path()))
        } else {
            None
        }
    }

    ///
    /// Reserves a path for a new file (this path is valid and won't be re-used by future calls but
    /// no files will exist here yet)
    ///
    fn create_new_path(&self) -> PathBuf {
        // Generate a filename
        let filename        = Uuid::new_v4().to_simple().to_string();
        let mut full_path   = self.root_path.clone();

        full_path.push(DATA_DIR);
        full_path.push(&filename);

        let update          = FileUpdate::NewFile(full_path.clone());

        // Add to the database
        let log_path         = full_path.clone();
        let mut filename_buf = PathBuf::new();
        filename_buf.push(filename);
        let _ = self.core.future(move |core| {
            core.log.log((Level::Info, format!("Created new file at `{}`", log_path.to_str().unwrap_or("<Missing path>"))));

            core.file_list.add_path(filename_buf.as_path()).unwrap();
            Box::pin(core.send_update(update))
        });

        // Result is the full path
        full_path
    }

    ///
    /// Re-orders the files so that `path` is displayed after `after` (or at the beginning if `after` is `None`)
    ///
    fn order_path_after(&self, path: &Path, after: Option<&Path>) {
        // Turn the paths into pathbufs
        let path    = PathBuf::from(path);
        let after   = after.map(|after| PathBuf::from(after));

        // Do nothing if trying to move the path after itself
        if after.as_ref() == Some(&path) {
            return;
        }

        // Generate the update
        let update  = FileUpdate::ChangedOrder(path.clone(), after.clone());

        // Update the file list
        let _ = self.core.future(move |core| {
            let after = after.as_ref();
            let after = after.map(|after| after.as_path());
            core.file_list.order_path_after(path.as_path(), after).unwrap();

            Box::pin(core.send_update(update))
        });
    }

    ///
    /// Updates or creates the display name associated with a particular path (which must be
    /// returned via get_all_xfiles: setting the name for a non-existent path will just
    /// result)
    ///
    fn set_display_name_for_path(&self, full_path: &Path, display_name: String) {
        let path = self.file_list_path(full_path);

        if let Some(path) = path {
            let update = FileUpdate::SetDisplayName(PathBuf::from(full_path), display_name.clone());

            let _ = self.core.future(move |core| {
                core.file_list.set_display_name_for_path(path.as_path(), &display_name).unwrap();
                Box::pin(core.send_update(update))
            });
        }
    }

    ///
    /// Returns a stream of updates indicating changes made to the file manager
    ///
    fn update_stream(&self) -> BoxStream<'static, FileUpdate> {
        // Get a subscription from the core
        let subscription = self.core.sync(|core| core.updates.subscribe());

        // Return to the caller
        Box::pin(subscription)
    }


    ///
    /// Removes a path from this manager and deletes the file that was found there
    ///
    fn delete_path(&self, full_path: &Path) {
        // Look up the path that we want to delete
        let path        = self.file_list_path(full_path);
        let full_path   = PathBuf::from(full_path);

        if let Some(path) = path {
            // Start deleting it if we find it
            let update = FileUpdate::RemovedFile(full_path.clone());

            let _ = self.core.future(move |core| {
                core.log.log((Level::Info, format!("Deleting file at path `{}`", full_path.to_str().unwrap_or("<Missing path>"))));

                // Delete from the file list
                core.file_list.remove_path(path.as_path()).unwrap();

                // Delete from disk
                if full_path.starts_with("/") && full_path.is_file() {
                    let result = fs::remove_file(full_path.as_path());
                    match result {
                        Ok(_)       => { },
                        Err(erm)    => { core.log.log((Level::Warn, format!("Failed to delete `{}`: {:?}", full_path.to_str().unwrap_or("<Missing path>"), erm))); }
                    }
                } else {
                    core.log.log((Level::Warn, format!("Not deleting `{}` (doesn't exist or path is in wrong format)", full_path.to_str().unwrap_or("<Missing path>"))));
                }

                // Notify that the file is gone
                Box::pin(core.send_update(update))
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor;

    #[test]
    fn create_new_path() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "create_new_path");
        let new_path    = test_files.create_new_path();

        assert!(new_path.components().count() > 3);
    }

    #[test]
    fn remove_created_path() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "remove_created_path");
        let num_files   = test_files.get_all_files().len();
        let new_path    = test_files.create_new_path();

        assert!(num_files+1 == test_files.get_all_files().len());

        test_files.delete_path(new_path.as_path());

        assert!(num_files == test_files.get_all_files().len());
    }

    #[test]
    fn retrieve_new_path_from_all_files() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "retrieve_new_path_from_all_files");

        let all_files_before    = test_files.get_all_files();
        let _new_path           = test_files.create_new_path();
        let all_files_after     = test_files.get_all_files();

        assert!(all_files_before.len()+1 == all_files_after.len());
    }

    #[test]
    fn new_paths_are_created_at_start() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "new_paths_are_created_at_start");
        let new_path1   = test_files.create_new_path();
        let new_path2   = test_files.create_new_path();

        let paths       = test_files.get_all_files();

        assert!(paths[0] == new_path2);
        assert!(paths[1] == new_path1);
    }

    #[test]
    fn move_path_after() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "move_path_after");
        let new_path1   = test_files.create_new_path();
        let new_path2   = test_files.create_new_path();

        test_files.order_path_after(new_path2.as_path(), Some(new_path1.as_path()));

        let paths       = test_files.get_all_files();

        assert!(paths[0] == new_path1);
        assert!(paths[1] == new_path2);
    }

    #[test]
    fn display_name_is_initially_none() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "display_name_is_initially_none");
        let new_path    = test_files.create_new_path();

        assert!(test_files.display_name_for_path(new_path.as_path()) == None);
    }

    #[test]
    fn set_alternative_display_name() {
        let test_files  = SqliteFileManager::new("app.flowbetween.test", "set_alternative_display_name");
        let new_path    = test_files.create_new_path();

        test_files.set_display_name_for_path(new_path.as_path(), "Test display name".to_string());
        assert!(test_files.display_name_for_path(new_path.as_path()) == Some("Test display name".to_string()));
    }

    #[test]
    fn will_send_updates_to_stream() {
        let test_files          = SqliteFileManager::new("app.flowbetween.test", "will_send_updates_to_stream");
        let mut update_stream   = test_files.update_stream();

        let new_path            = test_files.create_new_path();
        test_files.set_display_name_for_path(new_path.as_path(), "Another display name".to_string());

        executor::block_on(async {
            assert!(update_stream.next().await == Some(FileUpdate::NewFile(new_path.clone())));
            assert!(update_stream.next().await == Some(FileUpdate::SetDisplayName(new_path.clone(), "Another display name".to_string())));
        })
    }
}
