use ::serde::*;

use std::collections::*;
use std::fmt;
use std::ops::{Deref};
use std::sync::*;

/// Maps shape type IDs to their names
static SHAPE_TYPE_NAMES: LazyLock<Mutex<Vec<&'static str>>> = LazyLock::new(|| Mutex::new(vec![]));

/// Hashmap mapping shape type names to IDs
static SHAPE_TYPE_FOR_NAME: LazyLock<Mutex<HashMap<&'static str, usize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

///
/// Represents the type of a shape
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeType(usize);

///
/// Lazy version of the shape type that can be initialised statically
///
pub struct LazyShapeType {
    /// Used to store the value once we've looked it up
    val: OnceLock<ShapeType>,

    /// The shape type name we need to look up
    name: &'static str,
}

impl ShapeType {
    ///
    /// Creates a shape type with a known name
    ///
    pub fn new(shape_type_name: &str) -> Self {
        let shape_type_for_name = SHAPE_TYPE_FOR_NAME.lock().unwrap();

        // Look up the value in the list of known shape types
        if let Some(existing_id) = shape_type_for_name.get(&shape_type_name)
        {
            // Use the existing value if one exists
            Self(*existing_id)
        }
        else
        {
            // If one doesn't exist, create an &'static str from the shape type name and associate it with a new unique ID
            // Note we hold both locks here, so take care to always take them in the order 'shape_type_for_name', 'shape_type_names'
            let shape_type_name             = Box::leak(shape_type_name.to_string().into_boxed_str());

            let mut shape_type_for_name     = shape_type_for_name;
            let mut shape_type_names        = SHAPE_TYPE_NAMES.lock().unwrap();
            let new_id                      = shape_type_names.len();

            shape_type_names.push(shape_type_name);
            shape_type_for_name.insert(shape_type_name, new_id);

            Self(new_id)
        }
    }

    ///
    /// Returns the name of this shape type
    ///
    pub fn name(&self) -> &'static str {
        // Look up the name associated with this shape type when `new()` was called
        SHAPE_TYPE_NAMES.lock().unwrap()[self.0]
    }
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::new("app.flowbetween::shape")
    }
}

impl fmt::Debug for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShapeType::new(\"{}\")", self.name())
    }
}

impl From<&str> for ShapeType {
    #[inline]
    fn from(val: &str) -> Self {
        Self::new(val)
    }
}

impl Serialize for ShapeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the shape type name instead of the internal ID
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for ShapeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the shape type name and convert it to a ShapeType
        let name = String::deserialize(deserializer)?;
        Ok(ShapeType::new(&name))
    }
}

impl LazyShapeType {
    ///
    /// Creates a lazy shape type (value will be generated when needed)
    ///
    /// This can be used with static shape types - eg:
    ///
    /// ```
    /// # use flow_between::scenery::document::canvas::*;
    /// static MY_SHAPE_TYPE: LazyShapeType = LazyShapeType::new("flo2::my_shape_type");
    ///
    /// let shape_type = *MY_SHAPE_TYPE;
    /// ```
    ///
    pub const fn new(name: &'static str) -> Self {
        Self {
            val:    OnceLock::new(),
            name:   name,
        }
    }
}

impl Deref for LazyShapeType {
    type Target = ShapeType;

    fn deref(&self) -> &Self::Target {
        self.val.get_or_init(|| ShapeType::new(self.name))
    }
}
