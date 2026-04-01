//!
//! The animation serializer provides a way to convert animation structures to and from a machine-readable ASCII format
//! 
//! Animation structures are basically divided into two: edit log items describe actions that change an animation,
//! and layer data describes the elements/entities that make up an animation.
//! 
//! The custom ASCII format is used for compactness and speed over more verbose formats like JSON, as animations
//! can contain a lot of data.
//!

mod source;
mod target;

mod edit;
mod color;
mod vector;
mod time_path;
mod cache_type;
mod element_id;
mod fill_option;
mod drawing_style;
mod path_component;
mod brush_definition;
mod brush_properties;

pub use self::source::*;
pub use self::target::*;

pub use self::edit::*;
pub use self::color::*;
pub use self::vector::*;
pub use self::time_path::*;
pub use self::cache_type::*;
pub use self::element_id::*;
pub use self::fill_option::*;
pub use self::drawing_style::*;
pub use self::path_component::*;
pub use self::brush_definition::*;
pub use self::brush_properties::*;
