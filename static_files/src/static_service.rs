use std::collections::HashMap;
use std::rc::Rc;
use std::iter::FromIterator;

use hyper::{Method, StatusCode};
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Request, Response, Service};

extern crate hyper;
extern crate futures;
extern crate mime;

use super::static_file::*;

///
/// Service that supplies static files
///
pub struct StaticService {
    /// File found in a particular path
    file_for_path: HashMap<String, Rc<StaticFile>>
}

impl StaticService {
    pub fn new(files: Vec<StaticFile>) -> StaticService {
        let paths_and_files = files
                .into_iter()
                .map(|file| Rc::new(file))
                .flat_map(|file| file.valid_paths().into_iter().map(move |path| (path, file.clone())));

        StaticService {
            file_for_path: HashMap::from_iter(paths_and_files)
        }
    }
}

impl Service for StaticService {
    type Request    = Request;
    type Response   = Response;
    type Error      = hyper::Error;
    type Future     = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Request) -> Self::Future {
        // Prepare the response
        let mut response = Response::new();

        if req.method() == &Method::Get {
            // Try to retrieve a static file
            let static_file = self.file_for_path.get(req.path());

            if let Some(static_file) = static_file {
                // File exists

                // Get the MIME type and the file content
                let mime_type   = static_file.mime_type().parse::<mime::Mime>().unwrap();
                let content     = static_file.content();

                // Set up the response
                // TODO: the extra copy of the content may be unnecessary here
                response.set_status(StatusCode::Ok);
                response.headers_mut().set(ContentType(mime_type));
                response.headers_mut().set(ContentLength(content.len() as u64));
                response.set_body(Vec::from(static_file.content()));
            } else {
                // File not found
                response.set_status(StatusCode::NotFound);
            }
        } else {
            // Only GET is supported for static files
            response.set_status(StatusCode::BadRequest);
        }

        // Response is available immediately
        futures::future::ok(response)
    }
}