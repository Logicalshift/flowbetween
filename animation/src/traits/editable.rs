use std::ops::{Deref, DerefMut};

///
/// Represents an item that can be edited
///
pub trait Editable<T> {
    ///
    /// Opens this item for editing, if an editor is available
    ///
    fn open(&self) -> Option<Editor<T>>;
}

///
/// Represents an editor for type T
///
pub struct Editor<'a, T: ?Sized+'a> {
    /// The target that is being edited
    target: Box<'a+DerefMut<Target=T>>,
}

impl<'a, T: ?Sized+'a> Editor<'a, T> {
    ///
    /// Creates a new editor from something that can be dereferenced as the specified type
    ///
    pub fn new<Owner: 'a+DerefMut<Target=T>>(target: Owner) -> Editor<'a, T> {
        Editor { target: Box::new(target) }
    }
}

impl<'a, T: ?Sized+'a> Deref for Editor<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.target.deref()
    }
}

impl<'a, T: ?Sized+'a> DerefMut for Editor<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.target.deref_mut()
    }
}

///
/// Opens an editable object for editing if possible
/// 
pub fn edit<'a, EditorType>(editable: &'a Editable<EditorType>) -> Option<Editor<'a, EditorType>> {
    editable.open()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::*;

    #[test]
    fn can_create_editor_for_mutex() {
        let mutex = Mutex::new(1);

        {
            // Lock the mutex and turn the lock into an editor
            let mut editor = {
                let locked = mutex.lock().unwrap();
                Editor::new(locked)
            };

            // Can use the editor like it was the lock
            assert!(*editor == 1);
            *editor = 2;
        }

        // Updates stay afterwards
        assert!(*mutex.lock().unwrap() == 2);
    }

    struct TestEditable;

    impl Editable<i32> for TestEditable {
        fn open(&self) -> Option<Editor<i32>> { None }
    }

    impl Editable<bool> for TestEditable {
        fn open(&self) -> Option<Editor<bool>> { None }
    }

    #[test]
    fn can_have_multiple_editable_items() {
        let test = TestEditable;
        let edit_i32:Option<Editor<i32>>    = test.open();
        let edit_bool                       = edit::<bool>(&test);

        assert!(edit_i32.is_none());
        assert!(edit_bool.is_none());
    }
}
