use super::super::serializer::*;

use std::time::{Duration};

///
/// Storage/serialization structure used to represent the properties of a file
///
#[derive(Clone)]
pub struct FileProperties {
    /// The name of the animation
    pub name: String,

    /// The size of the canvas
    pub size: (f64, f64),

    /// The length of the animation
    pub duration: Duration,

    /// The length of a frame in the animation
    pub frame_length: Duration
}

impl Default for FileProperties {
    fn default() -> FileProperties {
        // Default is an unnamed 30fps animation
        FileProperties {
            name:           "".to_string(),
            size:           (1920.0, 1080.0),
            duration:       Duration::from_millis(1000 * 60 * 2),
            frame_length:   Duration::new(0, 33_333_333)
        }
    }
}

impl FileProperties {
    ///
    /// Serializes these file properties to a target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // Version 0 of the properties
        data.write_small_u64(0);

        data.write_str(&self.name);
        data.write_f64(self.size.0);
        data.write_f64(self.size.1);
        data.write_duration(self.duration);
        data.write_duration(self.frame_length);
    }

    ///
    /// Deserializes file properties from a target
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<FileProperties> {
        let mut result = FileProperties::default();

        match data.next_small_u64() {
            0 => {
                result.name             = data.next_string();
                result.size             = (data.next_f64(), data.next_f64());
                result.duration         = data.next_duration();
                result.frame_length     = data.next_duration();

                Some(result)
            }

            _ => None
        }
    }
}
