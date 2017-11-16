use ui::*;
use ui::canvas::*;

use iron::response::*;
use std::io::*;

///
/// Provides a WriteBody implementation for a canvas (writes the entire canvas up to now)
///
pub struct CanvasBody(Resource<Canvas>);

impl CanvasBody {
    pub fn new(canvas: &Resource<Canvas>) -> CanvasBody {
        CanvasBody(canvas.clone())
    }
}

impl WriteBody for CanvasBody {
    fn write_body(&mut self, res: &mut Write) -> Result<()> {
        // Stream everything that's ready from the canvas
        let canvas      = &*self.0;
        let drawing     = canvas.get_drawing();

        // Stream until there's nothing left
        for draw in drawing.into_iter() {
            // Encode this command
            let mut encoded = String::new();
            draw.encode_canvas(&mut encoded);
            encoded.push('\n');

            // Send to the stream and check for errors
            let res = res.write(encoded.as_bytes());
            if let Err(erm) = res {
                return Err(erm);
            }
        }

        Ok(())
    }
}