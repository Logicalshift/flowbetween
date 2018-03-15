use binding::*;
use animation::*;

///
/// Model representing the item that the user has selected
/// 
#[derive(Clone)]
pub struct SelectionModel {
    /// The currently selected element
    pub selected_element: BindRef<ElementId>,

    /// The binding for the selected element
    selected_element_binding: Binding<ElementId>
}

impl SelectionModel {
    ///
    /// Creates a new selection model
    /// 
    pub fn new() -> SelectionModel {
        // Create the binding for the selected element
        let selected_element_binding = bind(ElementId::Unassigned);

        SelectionModel {
            selected_element:           BindRef::new(&selected_element_binding),
            selected_element_binding:   selected_element_binding
        }
    }

    ///
    /// Select a particular element
    /// 
    pub fn select_element(&self, element: ElementId) {
        self.selected_element_binding.clone().set(element)
    }
}
