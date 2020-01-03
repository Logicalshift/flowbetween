use super::*;
use super::bytes_iterator::*;

use std::io::Read;

///
/// Represents an image whose data is stored in memory
///
pub struct InMemoryImageData {
    /// The bytes making up this stream
    bytes: Arc<Bytes>
}

impl InMemoryImageData {
    ///
    /// Creates a new image data object from a set of bytes
    ///
    pub fn new(bytes: Bytes) -> InMemoryImageData {
        InMemoryImageData {
            bytes: Arc::new(bytes)
        }
    }
}

impl From<Vec<u8>> for InMemoryImageData {
    fn from(bytes: Vec<u8>) -> InMemoryImageData {
        InMemoryImageData::new(Bytes::from(bytes))
    }
}

impl From<Bytes> for InMemoryImageData {
    fn from(bytes: Bytes) -> InMemoryImageData {
        InMemoryImageData {
            bytes: Arc::new(bytes)
        }
    }
}

impl From<Arc<Bytes>> for InMemoryImageData {
    fn from(bytes: Arc<Bytes>) -> InMemoryImageData {
        InMemoryImageData {
            bytes: bytes
        }
    }
}

impl<'a> From<&'a Arc<Bytes>> for InMemoryImageData {
    fn from(bytes: &'a Arc<Bytes>) -> InMemoryImageData {
        InMemoryImageData {
            bytes: Arc::clone(bytes)
        }
    }
}

impl ImageData for InMemoryImageData {
    fn read(&self) -> Box<dyn Read+Send> {
        Box::new(ImageStreamIterator::from(&self.bytes))
    }

    fn read_future(&self) -> BoxStream<'static, Bytes> {
        Box::pin(ImageStreamIterator::from(&self.bytes))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::prelude::*;
    use futures::executor;

    #[test]
    fn can_read_all_image_data() {
        let data_test       = InMemoryImageData::new(Bytes::from(vec![1,2,3,4,5,6]));
        let read_data       = data_test.read_future();
        let read_collect    = read_data.collect::<Vec<_>>();

        let bytes_back      = executor::block_on(read_collect);

        assert!(bytes_back == vec![Bytes::from(vec![1,2,3,4,5,6])]);
    }
}
