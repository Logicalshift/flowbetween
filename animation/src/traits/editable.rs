use std::ops::{Deref, DerefMut};
use super::edit::*;

///
/// Represents an item that can be edited
///
pub trait Editable<T: ?Sized> {
    ///
    /// Opens this item for editing, if an editor is available
    ///
    fn edit(&self) -> Option<Editor<T>>;

    ///
    /// Opens this item for reading, if a reader is available
    ///
    fn read(&self) -> Option<Reader<T>>;
}

///
/// Represents a reader for type T
/// 
pub struct Reader<'a, T: ?Sized+'a> {
    target: Box<'a+Deref<Target=T>>
}

///
/// Represents an editor for type T
///
pub struct Editor<'a, T: ?Sized+'a> {
    /// The target that is being edited
    target: Box<'a+DerefMut<Target=T>>,
}

impl<'a, T: ?Sized+'a> Reader<'a, T> {
    ///
    /// Creates a new reader from something that can be dereferenced as the specified type
    ///
    pub fn new<Owner: 'a+Deref<Target=T>>(target: Owner) -> Reader<'a, T> {
        Reader { target: Box::new(target) }
    }
}

impl<'a, T: ?Sized+'a> Editor<'a, T> {
    ///
    /// Creates a new editor from something that can be dereferenced as the specified type
    ///
    pub fn new<Owner: 'a+DerefMut<Target=T>>(target: Owner) -> Editor<'a, T> {
        Editor { target: Box::new(target) }
    }
}

impl<'a, T: ?Sized+'a> Deref for Reader<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.target.deref()
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

impl<T: ?Sized> Editable<T> for () {
    fn edit(&self) -> Option<Editor<T>> { None }
    fn read(&self) -> Option<Reader<T>> { None }
}

///
/// Opens an editable object for reading if possible
/// 
pub fn open_read<'a, EditorType: ?Sized>(editable: &'a Editable<EditorType>) -> Option<Reader<'a, EditorType>> {
    editable.read()
}

///
/// Opens an editable object for editing if possible
/// 
pub fn open_edit<'a, EditorType: ?Sized>(editable: &'a Editable<EditorType>) -> Option<Editor<'a, EditorType>> {
    editable.edit()
}

///
/// Convenience trait that makes it easier to edit an object that uses an EditLog
/// 
pub trait PerformEdits<Edit> {
    fn perform_edits(&self, edits: &[Edit]);
    fn set_pending(&self, pending: &[Edit]);
    fn commit_pending(&self);
}

impl<Edit, T: Editable<EditLog<Edit>>> PerformEdits<Edit> for T {
    #[inline]
    fn perform_edits(&self, edits: &[Edit]) {
        let mut editor = open_edit::<EditLog<Edit>>(self).unwrap();

        editor.set_pending(edits);
        editor.commit_pending();
    }

    #[inline]
    fn set_pending(&self, edits: &[Edit]) {
        let mut editor = open_edit::<EditLog<Edit>>(self).unwrap();

        editor.set_pending(edits);
    }

    #[inline]
    fn commit_pending(&self) {
        let mut editor = open_edit::<EditLog<Edit>>(self).unwrap();

        editor.commit_pending();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::*;

    #[test]
    fn can_create_reader_for_mutex() {
        let mutex = Mutex::new(1);

        {
            // Lock the mutex and turn the lock into a reader
            let reader = {
                let locked = mutex.lock().unwrap();
                Reader::new(locked)
            };

            // Can use the reader like it was the lock
            assert!(*reader == 1);
        }
    }

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
        fn edit(&self) -> Option<Editor<i32>> { None }
        fn read(&self) -> Option<Reader<i32>> { None }
    }

    impl Editable<bool> for TestEditable {
        fn edit(&self) -> Option<Editor<bool>> { None }
        fn read(&self) -> Option<Reader<bool>> { None }
    }

    #[test]
    fn can_have_multiple_editable_items() {
        let test = TestEditable;
        let edit_i32:Option<Editor<i32>>    = test.edit();
        let edit_bool                       = open_edit::<bool>(&test);

        assert!(edit_i32.is_none());
        assert!(edit_bool.is_none());
    }
}
