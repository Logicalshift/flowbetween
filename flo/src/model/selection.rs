use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::collections::HashSet;

///
/// Model representing the item that the user has selected
/// 
#[derive(Clone)]
pub struct SelectionModel {
    /// The list of selected elements
    pub selected_element: BindRef<Arc<HashSet<ElementId>>>,

    /// The binding for the selected element (used when updating)
    selected_element_binding: Binding<Arc<HashSet<ElementId>>>
}

impl SelectionModel {
    ///
    /// Creates a new selection model
    /// 
    pub fn new() -> SelectionModel {
        // Create the binding for the selected element
        let selected_element_binding = bind(Arc::new(HashSet::new()));

        SelectionModel {
            selected_element:           BindRef::new(&selected_element_binding),
            selected_element_binding:   selected_element_binding
        }
    }

    ///
    /// Adds a particular element to the selection
    /// 
    pub fn select(&self, element: ElementId) {
        // Not *ideal* because there's a race condition here
        let existing_selection = self.selected_element_binding.get();

        let mut new_selection = (*existing_selection).clone();
        new_selection.insert(element);
        self.selected_element_binding.set(Arc::new(new_selection));
    }

    ///
    /// Clears the current selection
    ///
    pub fn clear_selection(&self) {
        self.selected_element_binding.set(Arc::new(HashSet::new()));
    }
}
