use futures::stream::{BoxStream};
use bytes::Bytes;

use std::sync::*;
use std::io::Read;

///
/// Represents a static image
///
#[derive(Clone)]
pub enum Image {
    /// Represents an image containing PNG data
    Png(Arc<dyn ImageData>),

    /// Represents an image containing SVG data
    Svg(Arc<dyn ImageData>)
}

///
/// Trait implemented by things that can provide data for an image
///
pub trait ImageData : Send+Sync {
    /// Reads the raw data for this image
    fn read(&self) -> Box<dyn Read+Send>;

    /// Reads the raw data for this image
    fn read_future(&self) -> BoxStream<'static, Bytes>;
}

impl Image {
    ///
    /// Creates a new image from an RGBA buffer
    ///
    pub fn png_from_rgba_data(rgba: &[u8], width: u32, height: u32) -> Image {
        Image::Png(Arc::new(png::png_data_for_rgba(rgba, width, height)))
    }
}

mod inmemory;
mod static_data;
mod shortcuts;
mod png;
mod bytes_iterator;
pub use self::inmemory::*;
pub use self::static_data::*;
pub use self::shortcuts::*;
