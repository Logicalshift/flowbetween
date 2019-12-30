use std::io::{Read, Result};
use std::pin::*;
use std::sync::*;

use bytes::Bytes;
use futures::*;
use futures::task::{Poll, Context};

#[derive(Clone)]
pub struct ImageStreamIterator {
    /// The bytes in the image data
    bytes: Arc<Bytes>,

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
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<Option<Bytes>> {
        let max_to_read     = 100000;
        let pos             = self.pos;
        let len             = self.bytes.len();
        let num_to_read     = if pos+max_to_read > len { len-pos } else { max_to_read };

        if num_to_read > 0 {
            self.pos += num_to_read;
            Poll::Ready(Some(self.bytes.slice(pos..(pos+num_to_read))))
        } else {
            Poll::Ready(None)
        }
    }
}

impl Read for ImageStreamIterator {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Work out how many bytes to read
        let mut num_to_read = buf.len();
        if self.pos+num_to_read > self.bytes.len() {
            num_to_read = self.bytes.len()-self.pos;
        }

        // Copy to the buffer
        buf[..num_to_read].copy_from_slice(&self.bytes[self.pos..(self.pos+num_to_read)]);

        // Update the position
        self.pos += num_to_read;

        Ok(num_to_read)
    }
}

impl<'a> From<&'a Arc<Bytes>> for ImageStreamIterator {
    fn from(bytes: &'a Arc<Bytes>) -> ImageStreamIterator {
        ImageStreamIterator {
            bytes:  Arc::clone(bytes),
            pos:    0
        }
    }
}
