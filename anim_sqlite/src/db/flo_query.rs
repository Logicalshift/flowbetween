use super::db_enum::*;
use super::motion_path_type::*;

use flo_animation::*;
use std::time::Duration;

/* TODO: remove dependency on the Sqlite result type */
use rusqlite::*;

///
/// Entry read from the edit log
/// 
pub struct EditLogEntry {
    pub edit_id:                i64,
    pub edit_type:              EditLogType,
    pub layer_id:               Option<u64>,
    pub when:                   Option<Duration>,
    pub brush:                  Option<(i64, DrawingStyleType)>,
    pub brush_properties_id:    Option<i64>,
    pub element_id:             Option<i64>
}

///
/// Entry read from the colour table
/// 
pub struct ColorEntry {
    pub color_type:             ColorType,
    pub rgb:                    Option<(f64, f64, f64)>,
    pub hsluv:                  Option<(f64, f64, f64)>
}

///
/// Entry read from the brush table
/// 
pub struct BrushEntry {
    pub brush_type: BrushDefinitionType,
    pub ink_defn:   Option<(f64, f64, f64)>
}

///
/// Entry read from the brush properties table
/// 
pub struct BrushPropertiesEntry {
    pub size:       f64,
    pub opacity:    f64,
    pub color_id:   i64
}

///
/// Entry read from the vector element table
/// 
pub struct VectorElementEntry {
    pub element_id:             i64,
    pub element_type:           VectorElementType,
    pub when:                   Option<Duration>,
    pub brush:                  Option<(i64, DrawingStyleType)>,
    pub brush_properties_id:    Option<i64>,
    pub assigned_id:            ElementId
}

///
/// Entry read from the path element table
///
pub struct PathElementEntry {
    pub element_id:             i64,
    pub path_id:                i64,
    pub brush_id:               i64,
    pub brush_properties_id:    i64
}

///
/// Entry read from the time point table
/// 
#[derive(PartialEq)]
pub struct TimePointEntry {
    pub x:              f32,
    pub y:              f32,
    pub milliseconds:   f32
}

///
/// Entry read for a motion
/// 
#[derive(PartialEq)]
pub struct MotionEntry {
    pub motion_type:    MotionType,
    pub origin:         Option<(f32, f32)>,
}

///
/// Trait implemented by objects that can query an underlying store for FlowBetween
/// 
pub trait FloQuery {
    ///
    /// Finds the real layer ID and name for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<(i64, Option<String>)>;

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64, from: Duration, until: Duration) -> Result<Vec<Duration>>;

    ///
    /// Finds the nearest keyframe to the specified time in the specified layer
    /// 
    fn query_nearest_key_frame(&mut self, layer_id: i64, when: Duration) -> Result<Option<(i64, Duration)>>;

    ///
    /// Similar to query_nearest_key_frame except finds the previous and next keyframes instead
    /// 
    fn query_previous_and_next_key_frame(&mut self, layer_id: i64, when: Duration) -> Result<(Option<(i64, Duration)>, Option<(i64, Duration)>)>;

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64)>;

    ///
    /// Returns the total length of the animation
    /// 
    fn query_duration(&mut self) -> Result<Duration>;

    ///
    /// Returns the length of a frame in the animation
    /// 
    fn query_frame_length(&mut self) -> Result<Duration>;

    ///
    /// Returns the assigned layer IDs
    /// 
    fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>>;

    ///
    /// Retrieves the total number of entries in the edit log
    /// 
    fn query_edit_log_length(&mut self) -> Result<i64>;

    ///
    /// Retrieves a set of values from the edit log
    /// 
    fn query_edit_log_values(&mut self, from_index: i64, to_index: i64) -> Result<Vec<EditLogEntry>>;

    ///
    /// Queries the size associated with an edit log entry
    /// 
    fn query_edit_log_size(&mut self, edit_id: i64) -> Result<(f64, f64)>;

    ///
    /// Retrieves the raw points associated with a particular edit ID
    /// 
    fn query_edit_log_raw_points(&mut self, edit_id: i64) -> Result<Vec<RawPoint>>;

    ///
    /// Retrieves the ID of the path associated with the specified edit ID
    ///
    fn query_edit_log_path_id(&mut self, edit_id: i64) -> Result<i64>;

    ///
    /// Retrieves the string associated with a specific edit ID
    ///
    fn query_edit_log_string(&mut self, edit_id: i64, string_index: u32) -> Result<String>;

    ///
    /// Retrieves a colour with the specified ID
    /// 
    fn query_color(&mut self, color_id: i64) -> Result<ColorEntry>;

    ///
    /// Retrieves the brush with the specified ID
    /// 
    fn query_brush(&mut self, brush_id: i64) -> Result<BrushEntry>;

    ///
    /// Retrieves the brush properties with the specified ID
    /// 
    fn query_brush_properties(&mut self, brush_properties_id: i64) -> Result<BrushPropertiesEntry>;

    ///
    /// Retrieves the vector element with the specified ID
    ///
    fn query_vector_element(&mut self, id: i64) -> Result<VectorElementEntry>;

    ///
    /// Queries the vector elements that appear before a certain time in the specified keyframe
    /// 
    fn query_vector_keyframe_elements_before(&mut self, keyframe_id: i64, before: Duration) -> Result<Vec<VectorElementEntry>>;

    ///
    /// Queries the single most recent element of the specified type in the specified keyframe
    /// 
    fn query_most_recent_element_of_type(&mut self, keyframe_id: i64, before: Duration, element_type: VectorElementType) -> Result<Option<VectorElementEntry>>;

    ///
    /// Queries the brush points associated with a vector element
    /// 
    fn query_vector_element_brush_points(&mut self, element_id: i64) -> Result<Vec<BrushPoint>>;

    ///
    /// Queries the type of a single vector element
    /// 
    fn query_vector_element_type(&mut self, element_id: i64) -> Result<Option<VectorElementType>>;

    ///
    /// Queries a path element
    ///
    fn query_path_element(&mut self, element_id: i64) -> Result<Option<PathElementEntry>>;

    ///
    /// Queries the path components associated with a vector element
    ///
    fn query_path_components(&mut self, path_id: i64) -> Result<Vec<PathComponent>>;

    ///
    /// Queries the motion associated with a particular motion ID
    /// 
    fn query_motion(&mut self, motion_id: i64) -> Result<Option<MotionEntry>>;

    ///
    /// Queries the time points attached to a motion
    /// 
    fn query_motion_timepoints(&mut self, motion_id: i64, path_type: MotionPathType) -> Result<Vec<TimePointEntry>>;

    ///
    /// Retrieves the motions attached to a particular element ID
    /// 
    fn query_motion_ids_for_element(&mut self, assigned_element_id: i64) -> Result<Vec<i64>>;

    ///
    /// Retrieves the elements attached to a particular motion ID
    /// 
    fn query_element_ids_for_motion(&mut self, assigned_motion_id: i64) -> Result<Vec<i64>>;
}
