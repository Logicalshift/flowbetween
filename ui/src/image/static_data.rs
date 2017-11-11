use super::*;

use futures::*;

#[derive(Clone)]
struct ImageStreamIterator {
    /// The bytes in the image data
    bytes: &'static [u8],

    /// Current position
    pos: usize
}

impl Iterator for ImageStreamIterator {
    type Item=u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.bytes.len() {
            let res = self.bytes[self.pos];
            self.pos += 1;
            Some(res)
        } else {
            None
        }
    }
}

impl Stream for ImageStreamIterator {
    type Item = u8;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<u8>, ()> {
        use self::Async::*;

        Ok(Ready(self.next()))
    }
}

///
/// Represents an image whose data is stored in memory 
///
pub struct StaticImageData {
    /// The bytes making up this stream
    bytes: &'static [u8]
}

impl StaticImageData {
    pub fn new(bytes: &'static [u8]) -> StaticImageData {
        StaticImageData {
            bytes: bytes
        }
    }
}

impl ImageData for StaticImageData {
    /// Reads the raw data for this image
    fn read(&self) -> Box<Stream<Item=u8, Error=()>> {
        Box::new(ImageStreamIterator { bytes: self.bytes, pos: 0 })
    }
}
