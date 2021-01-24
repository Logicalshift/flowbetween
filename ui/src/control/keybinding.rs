use super::keypress::*;

use itertools::*;

use std::hash::{Hash, Hasher};
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

    ///
    /// Creates a CTRL+key binding
    ///
    pub fn ctrl(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierCtrl, key])
    }
}

impl Hash for KeyBinding {
    ///
    /// Keybindings are hashed by the ordered set of keys
    ///
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.keys.iter()
            .sorted()
            .for_each(|key| key.hash(state));
    }
}