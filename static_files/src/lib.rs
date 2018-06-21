//!
//! The static service provides static files for the flow-between
//!
#![warn(bare_trait_objects)]

extern crate sha2;

pub mod static_file;
pub mod static_service;

pub use static_file::*;
pub use static_service::*;

pub fn flowbetween_static_files() -> StaticService {
    StaticService::new(vec![
        StaticFile::new("/index.html",                      include_bytes!("../html/index.html")),

        StaticFile::new("/css/flowbetween.css",             include_bytes!("../css/flowbetween.css")),

        StaticFile::new("/js/flowbetween.js",               include_bytes!("../js/flowbetween.js")),
        StaticFile::new("/js/canvas.js",                    include_bytes!("../js/canvas.js")),
        StaticFile::new("/js/matrix.js",                    include_bytes!("../js/matrix.js")),
        StaticFile::new("/js/paint.js",                     include_bytes!("../js/paint.js")),
        StaticFile::new("/js/control.js",                   include_bytes!("../js/control.js")),

        StaticFile::new("/svg/controls/button.svg",         include_bytes!("../svg/controls/button.svg")),
        StaticFile::new("/png/Flo-Orb-small.png",           include_bytes!("../png/Flo-Orb-small.png")),

        StaticFile::new("/fonts/lato/Lato-Bold.woff2",      include_bytes!("../fonts/lato/Lato-Bold.woff2")),
        StaticFile::new("/fonts/lato/Lato-Regular.woff2",   include_bytes!("../fonts/lato/Lato-Regular.woff2")),
        StaticFile::new("/fonts/lato/Lato-Thin.woff2",      include_bytes!("../fonts/lato/Lato-Thin.woff2")),
        StaticFile::new("/fonts/lato/OFL.txt",              include_bytes!("../fonts/lato/OFL.txt")),
        StaticFile::new("/fonts/lato/README-WEB.txt",       include_bytes!("../fonts/lato/README-WEB.txt"))
    ])
}
