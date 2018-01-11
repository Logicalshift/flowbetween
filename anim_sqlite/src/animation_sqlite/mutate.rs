use super::*;

use animation::*;

impl MutableAnimation for SqliteAnimation {
    fn set_size(&mut self, size: (f64, f64)) {
        unimplemented!()
    }

    fn add_layer(&mut self, new_layer_id: u64) {
        unimplemented!()
    }

    fn remove_layer(&mut self, old_layer_id: u64) {
        unimplemented!()
    }

    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        unimplemented!()
    }
}
