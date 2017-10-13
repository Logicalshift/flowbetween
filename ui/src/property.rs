///
/// Represents a property (a control value that can either be a
/// constant or a viewmodel value)
///
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Property {
    Nothing,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),

    /// Property is bound to a value in the view model
    Bind(String)
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
