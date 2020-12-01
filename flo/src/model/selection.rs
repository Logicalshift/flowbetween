use super::frame::*;
use super::timeline::*;

use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::collections::HashSet;

///
/// Model representing the item that the user has selected
///
pub struct SelectionModel<Anim: Animation> {
    /// The list of selected elements
    pub selected_elements: BindRef<Arc<HashSet<ElementId>>>,

    /// The selected elements as they are ordered in the current frame (selected elements not in the current frame are excluded)
    pub selection_in_order: BindRef<Arc<Vec<ElementId>>>,

    /// The currently selected path (if this is set and the selected elements is empty, then the path has not yet been cut out from the layer)
    pub selected_path: Binding<Option<Arc<Path>>>,

    /// The binding for the selected element (used when updating)
    selected_elements_binding: Binding<Arc<HashSet<ElementId>>>,

    /// The animation this model is for
    animation: Arc<Anim>,

    /// The layers in the current frame
    layers: BindRef<Vec<FrameLayerModel>>,
}

impl<Anim: Animation> Clone for SelectionModel<Anim> {
    fn clone(&self) -> Self {
        SelectionModel {
            selected_elements:          self.selected_elements.clone(),
            selection_in_order:         self.selection_in_order.clone(),
            selected_path:              self.selected_path.clone(),
            selected_elements_binding:  self.selected_elements_binding.clone(),
            animation:                  self.animation.clone(),
            layers:                     self.layers.clone()
        }
    }
}

impl<Anim: 'static+Animation> SelectionModel<Anim> {
    ///
    /// Creates a new selection model
    ///
    pub fn new(animation: Arc<Anim>, frame_model: &FrameModel, timeline_model: &TimelineModel<Anim>) -> SelectionModel<Anim> {
        // Create the binding for the selected element
        let selected_elements_binding   = bind(Arc::new(HashSet::new()));
        let selected_elements           = BindRef::new(&selected_elements_binding);
        let selection_in_order          = Self::selection_in_order(selected_elements.clone(), frame_model, timeline_model);
        let selected_path               = bind(None);

        SelectionModel {
            selected_elements:          selected_elements,
            selected_elements_binding:  selected_elements_binding,
            selected_path:              selected_path,
            selection_in_order:         selection_in_order,
            animation:                  animation,
            layers:                     frame_model.layers.clone()
        }
    }

    ///
    /// Creates a binding of the selection in back-to-front order
    ///
    fn selection_in_order(selection: BindRef<Arc<HashSet<ElementId>>>, frame_model: &FrameModel, timeline_model: &TimelineModel<Anim>) -> BindRef<Arc<Vec<ElementId>>> {
        let frame               = frame_model.frame.clone();
        let invalidation_count  = timeline_model.canvas_invalidation_count.clone();

        let in_order = computed(move || {
            // Order needs to be recalculated if the frame is ever invalidated
            invalidation_count.get();

            // Vec where we store the in-order items
            let mut in_order = vec![];

            if let Some(frame) = frame.get() {
                // Fetch the un-ordered selection
                let selection = selection.get();

                // Fetch the elements in the frame
                if let Some(frame_elements) = frame.vector_elements() {
                    for element in frame_elements {
                        if selection.contains(&element.id()) {
                            in_order.push(element.id());
                        }
                    }
                }
            }

            Arc::new(in_order)
        });

        BindRef::new(&in_order)
    }

    ///
    /// Adds a particular element to the selection
    ///
    pub fn select(&self, element: ElementId) {
        // Not *ideal* because there's a race condition here
        let existing_selection = self.selected_elements_binding.get();

        let mut new_selection = (*existing_selection).clone();
        new_selection.insert(element);
        self.selected_elements_binding.set(Arc::new(new_selection));
    }

    ///
    /// Clears the current selection
    ///
    pub fn clear_selection(&self) {
        self.selected_elements_binding.set(Arc::new(HashSet::new()));
    }
}

impl<Anim: 'static+EditableAnimation> SelectionModel<Anim> {
    ///
    /// Performs a cut operation using the current selection path on the specified layer, adding the elements inside
    /// the path to the current selection.
    ///
    /// Usually you should check that there are no selected elements and a path set before calling this.
    ///
    pub fn cut_selection(&self, layer: u64) {

    }
}
