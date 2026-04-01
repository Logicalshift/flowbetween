use std::collections::HashMap;
use std::sync::Arc;
use std::iter::FromIterator;

use super::static_file::*;

///
/// Service that supplies static files
///
pub struct StaticService {
    /// File found in a particular path
    file_for_path: HashMap<String, Arc<StaticFile>>
}

impl StaticService {
    ///
    /// Creates a new static service
    ///
    pub fn new(files: Vec<StaticFile>) -> StaticService {
        let paths_and_files = files
                .into_iter()
                .map(|file| Arc::new(file))
                .flat_map(|file| file.valid_paths().into_iter().map(move |path| (path, file.clone())));

        StaticService {
            file_for_path: HashMap::from_iter(paths_and_files)
        }
    }

    ///
    /// Returns the file located at the specified path
    ///
    pub fn file_for_path(&self, path: &str) -> Option<Arc<StaticFile>> {
        if path.len() == 0 {
            self.file_for_path("/index.html")
        } else if path == "/" {
            self.file_for_path("/index.html")
        } else if path.chars().nth(0) != Some('/') {
            self.file_for_path(&format!("/{}", path))
        } else {
            self.file_for_path.get(path).cloned()
        }
    }
}
