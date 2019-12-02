use super::*;
use super::super::error::*;

use flo_animation::*;

use futures::*;
use std::sync::*;
use std::time::Duration;
use std::ops::Range;
use std::path::Path;

impl SqliteAnimation {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    ///
    pub fn retrieve_and_clear_error(&self) -> Option<SqliteAnimationError> {
        self.db.retrieve_and_clear_error()
    }

    ///
    /// Panics if this animation has reached an error condition
    ///
    pub fn panic_on_error(&self) {
        self.retrieve_and_clear_error().map(|erm| panic!("{:?}", erm));
    }
}

impl Animation for SqliteAnimation {
    #[inline]
    fn size(&self) -> (f64, f64) {
        self.db.size()
    }

    #[inline]
    fn get_layer_ids(&self) -> Vec<u64> {
        self.db.get_layer_ids()
    }

    fn duration(&self) -> Duration {
        self.db.duration()
    }

    fn frame_length(&self) -> Duration {
        self.db.frame_length()
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Arc<dyn Layer>> {
        // Try to retrieve the layer from the editor
        let layer = self.db.get_layer_with_id(layer_id);

        // Turn into a reader if it exists
        let layer = layer.map(|layer| {
            let layer_ref: Arc<dyn Layer> = Arc::new(layer);
            layer_ref
        });

        layer
    }

    fn get_num_edits(&self) -> usize {
        self.db.get_num_edits().unwrap_or(0)
    }

    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<dyn 'a+Stream<Item=AnimationEdit, Error=()>> {
        self.db.read_edit_log(range)
    }

    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        self
    }
}

impl FileAnimation for SqliteAnimation {
    fn open(path: &Path) -> SqliteAnimation {
        // TODO: error handling!

        if path.exists() {
            // Open an existing file
            Self::open_file(path).unwrap()
        } else {
            // Create a new file
            let animation = Self::new_with_file(path).unwrap();

            // Add a single layer and an initial keyframe
            animation.perform_edits(vec![
                AnimationEdit::SetSize(1980.0, 1080.0),
                AnimationEdit::AddNewLayer(0),
                AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0)))
            ]);

            animation
        }
    }
}
