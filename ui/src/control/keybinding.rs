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
    /// Creates a single key binding
    ///
    pub fn key(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![key])
    }

    ///
    /// Creates a ctrl+key binding
    ///
    pub fn ctrl(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierCtrl, key])
    }

    ///
    /// Creates a ctrl+alt+key binding
    ///
    pub fn ctrl_alt(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierCtrl, KeyPress::ModifierAlt, key])
    }

    ///
    /// Creates a ctrl+shift+key binding
    ///
    pub fn ctrl_shift(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierCtrl, KeyPress::ModifierShift, key])
    }

    ///
    /// Creates a ctrl+alt+shift+key binding
    ///
    pub fn ctrl_alt_shift(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierCtrl, KeyPress::ModifierAlt, KeyPress::ModifierShift, key])
    }

    ///
    /// Creates a meta+key binding
    ///
    pub fn meta(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierMeta, key])
    }

    ///
    /// Creates a meta+alt+key binding
    ///
    pub fn meta_alt(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierMeta, KeyPress::ModifierAlt, key])
    }

    ///
    /// Creates a meta+shift+key binding
    ///
    pub fn meta_shift(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierMeta, KeyPress::ModifierShift, key])
    }

    ///
    /// Creates a meta+alt+shift+key binding
    ///
    pub fn meta_alt_shift(key: KeyPress) -> KeyBinding {
        Self::hold_down_keys(vec![KeyPress::ModifierMeta, KeyPress::ModifierAlt, KeyPress::ModifierShift, key])
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