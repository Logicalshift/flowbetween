use std::sync::*;
use futures::stream::*;

///
/// Represents a static image
///
pub enum Image {
    /// Represents an image containing PNG data
    Png(Arc<ImageData>),

    /// Represents an image containing SVG data
    Svg(Arc<ImageData>)
}

///
/// Trait implemented by things that can provide data for an image
///
pub trait ImageData : Send+Sync {
    /// Reads the raw data for this image
    fn read(&self) -> Box<Stream<Item=u8, Error=()>>;
}

mod inmemory;
pub use self::inmemory::*;
