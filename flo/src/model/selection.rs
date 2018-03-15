use binding::*;
use animation::*;

///
/// Model representing the item that the user has selected
/// 
#[derive(Clone)]
pub struct SelectionModel {
    /// The currently selected element
    pub selected_element: BindRef<Vec<ElementId>>,

    /// The binding for the selected element
    selected_element_binding: Binding<Vec<ElementId>>
}

impl SelectionModel {
    ///
    /// Creates a new selection model
    /// 
    pub fn new() -> SelectionModel {
        // Create the binding for the selected element
        let selected_element_binding = bind(vec![]);

        SelectionModel {
            selected_element:           BindRef::new(&selected_element_binding),
            selected_element_binding:   selected_element_binding
        }
    }

    ///
    /// Adds a particular element to the selection
    /// 
    pub fn select(&self, element: ElementId) {
        let existing_selection = self.selected_element_binding.get();

        let mut new_selection = existing_selection;
        new_selection.push(element);
        self.selected_element_binding.clone().set(new_selection);
    }

    ///
    /// Clears the current selection
    /// 
    pub fn clear_selection(&self) {
        self.selected_element_binding.clone().set(vec![]);
    }
}
