use super::super::model::*;

use flo_binding::*;

///
/// Model representing the state of the selection tools
///
#[derive(Clone)]
pub struct SelectToolModel {
    /// Tracks the number of items that are currently selected by the tool
    pub num_elements_selected: BindRef<u64>,

    /// True if any items have been selected
    pub anything_selected: BindRef<bool>
}

impl SelectToolModel {
    ///
    /// Creates a new SelectModel
    ///
    pub fn new(selection_model: &SelectionModel) -> SelectToolModel {
        // Binding tracking the number of selected elements
        let selection               = selection_model.selected_elements.clone();
        let num_elements_selected   = computed(move || selection.get().len() as u64);

        // Binding tracking if anything at all is selected
        let selection               = selection_model.selected_elements.clone();
        let anything_selected       = computed(move || selection.get().len() > 0);

        // Create the model
        SelectToolModel {
            num_elements_selected:  BindRef::new(&num_elements_selected),
            anything_selected:      BindRef::new(&anything_selected)
        }
    }
}
