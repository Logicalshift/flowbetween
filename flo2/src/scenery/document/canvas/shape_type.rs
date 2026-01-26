use ::serde::*;

use std::collections::*;
use std::sync::*;

/// Maps shape type IDs to their names
static SHAPE_TYPE_NAMES: LazyLock<Mutex<Vec<&'static str>>> = LazyLock::new(|| Mutex::new(vec![]));

/// Hashmap mapping shape type names to IDs
static SHAPE_TYPE_FOR_NAME: LazyLock<Mutex<HashMap<&'static str, usize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

///
/// Represents the type of a shape
///
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ShapeType(usize);
