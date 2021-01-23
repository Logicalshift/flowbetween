use super::keypress::*;

use std::collections::{HashSet};

///
/// Describes a keyboard binding
///
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct KeyBinding {
    /// The set of keys that should be held down to trigger this action
    pub keys: HashSet<KeyPress>
}

impl KeyBinding {
    ///
    /// Creates a keybinding that consists of holding down the specified keys
    ///
    pub fn hold_down_keys<KeyIter: IntoIterator>(keys: KeyIter) -> KeyBinding 
    where KeyIter::Item: Into<KeyPress> {
        KeyBinding { 
            keys: keys.into_iter().map(|key| key.into()).collect::<HashSet<KeyPress>>() 
        }
    }
}
