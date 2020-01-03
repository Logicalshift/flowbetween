use flo_static_files::*;

use actix_web::*;
use actix_web::http::Method;
use futures::future;
use futures::future::{LocalBoxFuture};

///
/// Creates the standard static file handler
///
pub fn flowbetween_static_file_handler() -> impl Fn(HttpRequest) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    static_file_handler(flowbetween_static_files())
}

///
/// Creates a handler for serving static files from a service
///
pub fn static_file_handler(static_files: StaticService) -> impl Fn(HttpRequest) -> LocalBoxFuture<'static, Result<HttpResponse, Error>> {
    move |req: HttpRequest| {
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
                let found           = HttpResponse::Ok()
                    .header(http::header::ETAG, etag)
                    .header(http::header::CONTENT_TYPE, content_type)
                    .header(http::header::CONTENT_LENGTH, format!("{}", file.content().len()))
                    .header(http::header::CACHE_CONTROL, "public, max-age=60")
                    .body(Vec::from(file.content()));

                Box::pin(future::ok(found))
            } else if req.method() == &Method::HEAD {
                // Just the headers
                let found           = HttpResponse::Ok()
                    .header(http::header::ETAG, etag)
                    .header(http::header::CONTENT_TYPE, content_type)
                    .header(http::header::CONTENT_LENGTH, format!("{}", file.content().len()))
                    .header(http::header::CACHE_CONTROL, "public, max-age=60")
                    .finish();

                Box::pin(future::ok(found))
            } else {
                // Unsupported method
                let not_supported = future::ok(HttpResponse::MethodNotAllowed().body("Method not allowed"));
                Box::pin(not_supported)
            }
        } else {
            // File does not exist
            let not_found = future::ok(HttpResponse::NotFound().body("Not found"));
            Box::pin(not_found)
        }
    }
}
