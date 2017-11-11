use super::*;

use futures::*;

#[derive(Clone)]
struct ImageStreamIterator {
    /// The bytes in the image data
    bytes: Arc<Vec<u8>>,

    /// Current position
    pos: usize
}

impl Iterator for ImageStreamIterator {
    type Item=u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.bytes.len() {
            let res = (*self.bytes)[self.pos];
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
pub struct InMemoryImageData {
    /// The bytes making up this stream
    bytes: Arc<Vec<u8>>
}

impl InMemoryImageData {
    pub fn new(bytes: Vec<u8>) -> InMemoryImageData {
        InMemoryImageData {
            bytes: Arc::new(bytes)
        }
    }
}

impl ImageData for InMemoryImageData {
    /// Reads the raw data for this image
    fn read(&self) -> Box<Stream<Item=u8, Error=()>> {
        Box::new(ImageStreamIterator { bytes: self.bytes.clone(), pos: 0 })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor;

    #[test]
    fn can_read_all_image_data() {
        let data_test       = InMemoryImageData::new(vec![1,2,3,4,5,6]);
        let read_data       = data_test.read();
        let read_collect    = read_data.collect();

        let bytes_back      = executor::spawn(read_collect).wait_future().unwrap();

        assert!(bytes_back == vec![1,2,3,4,5,6]);
    }
}