//!
//! The static service provides static files for the flow-between
//!

extern crate iron;
extern crate sha2;

pub mod static_file;
pub mod static_service;

pub use static_file::*;
pub use static_service::*;

pub fn flowbetween_static_files() -> StaticService {
    StaticService::new(vec![
        StaticFile::new("/index.html",          include_bytes!("../html/index.html")),

        StaticFile::new("/css/flowbetween.css", include_bytes!("../css/flowbetween.css")),

        StaticFile::new("/js/flowbetween.js",   include_bytes!("../js/flowbetween.js")),
    ])
}
