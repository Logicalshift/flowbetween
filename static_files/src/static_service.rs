use std::collections::HashMap;
use std::sync::Arc;
use std::iter::FromIterator;

use iron::*;

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
}

impl Handler for StaticService {
    ///
    /// Serves a static file as a request (no caching)
    ///
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Canonicalize the path
        let mut path_components = vec![];

        for component in req.url.path().iter() {
            if *component == "." {
                // Ignore
            } else if *component == "" {
                // Also ignore; note that if this is at the end the final path should have a '/' at the end
            } else if *component == ".." {
                // Up a level
                if path_components.len() > 0 {
                    path_components.pop();
                }
            } else {
                path_components.push(String::from(*component));
            }
        }

        // Fold into a path string
        let mut path_string = path_components.iter()
            .fold(String::from("/"), |so_far, next_item| so_far + "/" + next_item);

        if req.url.path().len() > 1 && req.url.path().last() == Some(&"") {
            path_string.push('/');
        }

        // Look up the file
        let file = self.file_for_path.get(&path_string);

        if let Some(file) = file {
            // File can handle it
            file.handle(req)
        } else {
            // 404 response
            Ok(Response::with((status::NotFound)))
        }
    }
}