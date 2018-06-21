use super::*;
use super::bytes_iterator::*;

use std::io::Read;

///
/// Represents an image whose data is stored in memory 
///
pub struct StaticImageData {
    /// The bytes making up this stream
    bytes: Arc<Bytes>
}

impl StaticImageData {
    pub fn new(bytes: &'static [u8]) -> StaticImageData {
        StaticImageData {
            bytes: Arc::new(Bytes::from_static(bytes))
        }
    }
}

impl ImageData for StaticImageData {
    fn read(&self) -> Box<dyn Read+Send> {
        Box::new(ImageStreamIterator::from(&self.bytes))
    }

    fn read_future(&self) -> Box<dyn Stream<Item=Bytes, Error=()>> {
        Box::new(ImageStreamIterator::from(&self.bytes))
    }
}
