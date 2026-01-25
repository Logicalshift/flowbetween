use ::serde::*;

use std::collections::*;
use std::ops::{Deref};
use std::sync::*;

///
/// Identifier for a canvas property
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasPropertyId(usize);

///
/// Trait implemented by types that can be converted to a property
///
pub trait ToCanvasProperties : Sized {
    /// Returns the properties that can represent this value
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)>;

    /// The properties that need to be set on something for it to have this value associated with it
    fn used_properties() -> Vec<CanvasPropertyId>;

    /// Creates this value if possible from the properties set in the iterator
    fn from_properties<'a>(&'a self, properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self>;
}

///
/// Lazy version of the canvas property ID that can be initialised statically
///
pub struct LazyCanvasPropertyId {
    /// Used to store the value once we've looked it up
    val: OnceLock<CanvasPropertyId>,

    /// The property name we need to look up
    name: &'static str,
}

///
/// Value of a specific property set on a shape, layer or brush
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum CanvasProperty {
    /// Property with a single float value
    Float(f64),

    /// Property with a single integer value
    Int(i64),

    /// Property with a value that's a floating point number
    FloatList(Vec<f64>),

    /// Property with a value that's a list of integers
    IntList(Vec<i64>),

    /// Property with a value that's a series of bytes
    ByteList(Vec<u8>),
}

/// Maps property IDs to their names
static PROPERTY_NAMES: LazyLock<Mutex<Vec<&'static str>>> = LazyLock::new(|| Mutex::new(vec![]));

/// Hashmap mapping property names to IDs
static PROPERTY_FOR_NAME: LazyLock<Mutex<HashMap<&'static str, usize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

impl CanvasPropertyId {
    ///
    /// Creates a property ID with a known name
    ///
    pub fn new(property_name: &str) -> Self {
        let property_for_name   = PROPERTY_FOR_NAME.lock().unwrap();

        // Look up the value in the list of known property IDs
        if let Some(existing_id) = property_for_name.get(&property_name)
        {
            // Use the existing value if one exists
            Self(*existing_id)
        }
        else
        {
            // If one doesn't exist, create an &'static str from the property name and associate it with a new unique ID, then generate the property from that
            // Note we hold both locks here, so take care to always take them in the order 'property_for_name', 'property_names'
            let property_name           = Box::leak(property_name.to_string().into_boxed_str());

            let mut property_for_name   = property_for_name;
            let mut property_names      = PROPERTY_NAMES.lock().unwrap();
            let new_id                  = property_names.len();

            property_names.push(property_name);
            property_for_name.insert(property_name, new_id);

            Self(new_id)
        }
    }

    ///
    /// Returns the name of this property
    ///
    pub fn name(&self) -> &'static str {
        // Look up the name associated with this property when `new()` was called
        PROPERTY_NAMES.lock().unwrap()[self.0]
    }
}

impl LazyCanvasPropertyId {
    ///
    /// Creates a lazy canvas property (ID value will be generated when needed)
    ///
    /// This can be used with static properties - eg:
    ///
    /// ```
    /// #use flo2::scenery::document::canvas::*;
    /// static MY_PROPERTY: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flo2::my_property");
    ///
    /// let property_id = *MY_PROPERTY;
    /// ```
    ///
    pub const fn new(name: &'static str) -> Self {
        Self {
            val:    OnceLock::new(),
            name:   name,
        }
    }
}

impl Deref for LazyCanvasPropertyId {
    type Target = CanvasPropertyId;

    fn deref(&self) -> &Self::Target {
        self.val.get_or_init(|| CanvasPropertyId::new(self.name))
    }
}

impl From<&str> for CanvasPropertyId {
    ///
    /// Creates a property ID from a 
    ///
    #[inline]
    fn from(val: &str) -> Self {
        Self::new(val)
    }
}