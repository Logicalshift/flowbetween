use flo_static_files::*;

use actix_web::*;
use actix_web::http::*;
use actix_web::dev::{Handler, AsyncResult};
use futures::future;

///
/// Creates the standard static file handler
/// 
pub fn flowbetween_static_file_handler<TState>() -> impl Handler<TState> {
    static_file_handler(flowbetween_static_files())
}

///
/// Creates a handler for serving static files from a service
///
pub fn static_file_handler<TState>(static_files: StaticService) -> impl Handler<TState> {
    move |req: &HttpRequest<TState>| {
        // The tail specifies the file
        let tail = req.match_info().get("tail");

        // Find the file at this path
        let file = tail.and_then(|tail| static_files.file_for_path(tail));

        if let Some(file) = file {
            // File exists
            let content_type    = file.mime_type().to_string();
            let etag            = file.etag();

            // Found a file
            if req.method() == &Method::GET {
                // Append the body and return
                let found           = req.build_response(StatusCode::OK)
                    .header(http::header::ETAG, etag)
                    .header(http::header::CONTENT_TYPE, content_type)
                    .header(http::header::CONTENT_LENGTH, format!("{}", file.content().len()))
                    .header(http::header::CACHE_CONTROL, "public, max-age=60")
                    .body(Vec::from(file.content()));

                AsyncResult::async(Box::new(future::ok(found)))
            } else if req.method() == &Method::HEAD {
                // Just the headers
                let found           = req.build_response(StatusCode::OK)
                    .header(http::header::ETAG, etag)
                    .header(http::header::CONTENT_TYPE, content_type)
                    .header(http::header::CONTENT_LENGTH, format!("{}", file.content().len()))
                    .header(http::header::CACHE_CONTROL, "public, max-age=60")
                    .finish();

                AsyncResult::async(Box::new(future::ok(found)))
            } else {
                // Unsupported method
                let not_supported = future::ok(req.build_response(StatusCode::METHOD_NOT_ALLOWED).body("Method not allowed"));
                AsyncResult::async(Box::new(not_supported))
            }
        } else {
            // File does not exist
            let not_found = future::ok(req.build_response(StatusCode::NOT_FOUND).body("Not found"));
            AsyncResult::async(Box::new(not_found))
        }
    }
}
