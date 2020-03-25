use super::file_model::*;

use ::desync::*;

use std::sync::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

struct OpenFileStoreCore<Model: FileModel> {
    /// The object that can load files for this file store
    loader: Arc<Model::Loader>,

    /// The files that are open in this store
    open_files: HashMap<PathBuf, Arc<Model>>
}

///
/// Manages the open files for a particular model
///
pub struct OpenFileStore<Model: FileModel> {
    /// The core of this store
    core: Desync<OpenFileStoreCore<Model>>
}

impl<Model: 'static+FileModel> OpenFileStore<Model> {
    ///
    /// Creates a new open file store
    ///
    pub fn new(loader: Arc<Model::Loader>) -> OpenFileStore<Model> {
        let core = OpenFileStoreCore {
            loader:     loader,
            open_files: HashMap::new()
        };

        OpenFileStore {
            core: Desync::new(core)
        }
    }

    ///
    /// Opens the shared data for a particular path
    ///
    pub fn open_shared(&self, path: &Path) -> Arc<Model> {
        // Fetch the shared data for this path if it's already loaded, or create a new set by opening the file if not
        let shared = self.core.sync(|core| {
            let loader      = Arc::clone(&core.loader);
            let path_buf    = PathBuf::from(path);

            Arc::clone(core.open_files.entry(path_buf)
                .or_insert_with(|| Arc::new(Model::open(loader, path))))
        });

        shared
    }

    ///
    /// Removes the shared data for a file if there are no remaining references
    ///
    pub fn close_shared(&self, path: &Path) {
        let path = PathBuf::from(path);

        // Remove the shared data from the core if the reference count gets low enough
        self.core.desync(move |core| {
            let ref_count = core.open_files.get(&path).map(|reference| Arc::strong_count(reference)).unwrap_or(0);

            if ref_count == 1 {
                // The only reference is the one in this store: finish closing the file
                core.open_files.remove(&path);
            }
        })
    }
}
