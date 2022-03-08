use crate::traits::*;
use crate::editor::element_wrapper::*;

use std::time::{Duration};
use std::collections::{HashMap};

///
/// Represents a keyframe read from storage
///
pub struct StorageKeyFrame {
    pub start_time: Duration,
    pub end_time:   Duration,
    pub elements:   HashMap<ElementId, ElementWrapper>
}
