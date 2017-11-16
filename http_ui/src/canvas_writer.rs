use ui::*;
use ui::canvas::*;

use iron::response::*;
use futures::executor;
use futures::executor::Notify;
use futures::*;
use std::sync::*;
use std::io::*;

///
/// Provides a WriteBody implementation for a canvas (writes the entire canvas up to now)
///
pub struct WriteCanvas(Resource<Canvas>);

impl WriteCanvas {
    pub fn new(canvas: &Resource<Canvas>) -> WriteCanvas {
        WriteCanvas(canvas.clone())
    }
}

struct DontNotify;

impl Notify for DontNotify {
    fn notify(&self, _id: usize) { }
}

impl WriteBody for WriteCanvas {
    fn write_body(&mut self, res: &mut Write) -> Result<()> {
        // Stream everything that's ready from the canvas
        let canvas      = &*self.0;
        let stream      = canvas.stream();
        let mut stream  = executor::spawn(stream);

        // Stream until there's nothing left
        let dont_notify = Arc::new(DontNotify);
        while let Ok(Async::Ready(Some(draw))) = stream.poll_stream_notify(&dont_notify, 0) {
            // Encode this comment
            let mut encoded = String::new();
            draw.encode_canvas(&mut encoded);

            // Send to the stream and check for errors
            let res = res.write(encoded.as_bytes());
            if let Err(erm) = res {
                return Err(erm);
            }
        }

        Ok(())
    }
}