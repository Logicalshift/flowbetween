use super::db_enum::*;

use animation::*;
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
    pub brush:                  Option<(u64, DrawingStyleType)>,
    pub brush_properties_id:    Option<u64>
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
/// Trait implemented by objects that can query an underlying store for FlowBetween
/// 
pub trait FloQuery {
    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<i64>;

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64) -> Result<Vec<Duration>>;

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64)>;

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
}
