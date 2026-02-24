use super::property::*;

use ::serde::*;

/// Property used to describe the width of the canvas
pub static PROP_CANVAS_WIDTH: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::canvas_width");

/// Property used to describe the height of the canvas
pub static PROP_CANVAS_HEIGHT: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::canvas_height");

/// Property used to describe the length of time of a single frame in the canvas
pub static PROP_CANVAS_TIME_PER_FRAME: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::canvas_frame_time");

/// Size of the canvas
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DocumentSize { pub width: f64, pub height: f64 }

/// Time per frame of an animation, measured in seconds
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DocumentTimePerFrame(pub f64);

impl ToCanvasProperties for DocumentSize {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![
            (*PROP_CANVAS_WIDTH,  CanvasProperty::Float(self.width as _)),
            (*PROP_CANVAS_HEIGHT, CanvasProperty::Float(self.height as _)),
        ]
    }
}

impl FromCanvasProperties for DocumentSize {
    fn used_properties() -> Vec<CanvasPropertyId> {
        vec![*PROP_CANVAS_WIDTH, *PROP_CANVAS_HEIGHT]
    }

    fn from_properties<'a>(properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut width  = None;
        let mut height = None;

        for (prop_id, prop_val) in properties {
            if *prop_id == *PROP_CANVAS_WIDTH       { if let CanvasProperty::Float(v) = prop_val { width  = Some(*v as f64); } }
            else if *prop_id == *PROP_CANVAS_HEIGHT { if let CanvasProperty::Float(v) = prop_val { height = Some(*v as f64); } }
        }

        Some(DocumentSize { width: width?, height: height? })
    }
}

impl ToCanvasProperties for DocumentTimePerFrame {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![
            (*PROP_CANVAS_TIME_PER_FRAME, CanvasProperty::Float(self.0 as _)),
        ]
    }
}

impl FromCanvasProperties for DocumentTimePerFrame {
    fn used_properties() -> Vec<CanvasPropertyId> {
        vec![*PROP_CANVAS_TIME_PER_FRAME]
    }

    fn from_properties<'a>(properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut time_per_frame = None;

        for (prop_id, prop_val) in properties {
            if *prop_id == *PROP_CANVAS_TIME_PER_FRAME {
                if let CanvasProperty::Float(v) = prop_val { time_per_frame = Some(*v as f64); }
            }
        }

        Some(DocumentTimePerFrame(time_per_frame?))
    }
}

