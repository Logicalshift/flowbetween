///
/// Represents a property (a control value that can either be a
/// constant or a viewmodel value)
///
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Property {
    Nothing,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),

    /// Property is bound to a value in the view model
    Bind(String)
}

///
/// Represents the value of a property
///
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PropertyValue {
    Nothing,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
}

impl Property {
    ///
    /// Returns a property bound to a value in the view model
    ///
    #[inline]
    pub fn bound(value_name: &str) -> Property {
        Property::Bind(value_name.to_string())
    }
}

impl PropertyValue {
    ///
    /// Returns the string value of this property, if it is one
    ///
    pub fn string(&self) -> Option<String> {
        if let &PropertyValue::String(ref result) = self {
            Some(result.clone())
        } else {
            None
        }
    }

    ///
    /// Returns the string value of this property, if it is one
    ///
    pub fn str(&self) -> Option<&str> {
        if let &PropertyValue::String(ref result) = self {
            Some(&*result)
        } else {
            None
        }
    }

    ///
    /// Returns the boolean value of this property, if it is one
    ///
    pub fn to_bool(&self) -> Option<bool> {
        if let &PropertyValue::Bool(result) = self {
            Some(result)
        } else {
            None
        }
    }

    ///
    /// Returns the floating point value of this property, if it is one
    ///
    pub fn to_f32(&self) -> Option<f32> {
        if let &PropertyValue::Float(result) = self {
            Some(result as f32)
        } else {
            None
        }
    }

    ///
    /// Returns the floating point value of this property, if it is one
    ///
    pub fn to_f64(&self) -> Option<f64> {
        if let &PropertyValue::Float(result) = self {
            Some(result)
        } else {
            None
        }
    }
}

impl From<f64> for Property {
    fn from(val: f64) -> Property {
        Property::Float(val)
    }
}

impl From<f32> for Property {
    fn from(val: f32) -> Property {
        Property::Float(val as f64)
    }
}

impl From<i32> for Property {
    fn from(val: i32) -> Property {
        Property::Int(val)
    }
}

impl From<bool> for Property {
    fn from(val: bool) -> Property {
        Property::Bool(val)
    }
}

impl<'a> From<&'a str> for Property {
    fn from(val: &'a str) -> Property {
        Property::String(val.to_string())
    }
}

impl From<String> for Property {
    fn from(val: String) -> Property {
        Property::String(val)
    }
}

impl ToString for Property {
    fn to_string(&self) -> String {
        match self {
            &Property::Nothing              => String::from("<<nothing>>"),
            &Property::Bool(ref b)          => b.to_string(),
            &Property::Int(ref i)           => i.to_string(),
            &Property::Float(ref f)         => f.to_string(),
            &Property::String(ref s)        => s.clone(),

            &Property::Bind(ref binding)    => format!("<<bound to {}>>", binding)
        }
    }
}

impl ToString for PropertyValue {
    fn to_string(&self) -> String {
        match self {
            &PropertyValue::Nothing         => String::from("<<nothing>>"),
            &PropertyValue::Bool(ref b)     => b.to_string(),
            &PropertyValue::Int(ref i)      => i.to_string(),
            &PropertyValue::Float(ref f)    => f.to_string(),
            &PropertyValue::String(ref s)   => s.clone()
        }
    }
}

///
/// Trait for types that can be converted to a property
///
pub trait ToProperty {
    fn to_property(self) -> Property;
}

impl<'a> ToProperty for &'a () {
    fn to_property(self) -> Property {
        Property::Nothing
    }
}

impl<'a> ToProperty for &'a str {
    fn to_property(self) -> Property {
        Property::String(String::from(self))
    }
}

impl<'a> ToProperty for &'a String {
    fn to_property(self) -> Property {
        Property::String(self.clone())
    }
}

impl<'a> ToProperty for &'a i32 {
    fn to_property(self) -> Property {
        Property::Int(*self)
    }
}

impl<'a> ToProperty for &'a f32 {
    fn to_property(self) -> Property {
        Property::Float((*self) as f64)
    }
}

impl<'a> ToProperty for &'a f64 {
    fn to_property(self) -> Property {
        Property::Float(*self)
    }
}

impl<'a> ToProperty for &'a PropertyValue {
    fn to_property(self) -> Property {
        match self {
            &PropertyValue::Nothing         => Property::Nothing,
            &PropertyValue::Bool(ref b)     => Property::Bool(*b),
            &PropertyValue::Int(ref i)      => Property::Int(*i),
            &PropertyValue::Float(ref f)    => Property::Float(*f),
            &PropertyValue::String(ref s)   => Property::String(s.clone())
        }
    }
}

impl From<Property> for Option<PropertyValue> {
    fn from(prop: Property) -> Option<PropertyValue> {
        match prop {
            Property::Nothing       => Some(PropertyValue::Nothing),
            Property::Bool(b)       => Some(PropertyValue::Bool(b)),
            Property::Int(i)        => Some(PropertyValue::Int(i)),
            Property::Float(f)      => Some(PropertyValue::Float(f)),
            Property::String(ref s) => Some(PropertyValue::String(s.clone())),
            Property::Bind(_)       => None
        }
    }
}
