use super::frame::*;
use super::timeline::*;

use flo_binding::*;
use flo_animation::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::description::*;

use std::sync::*;
use std::time::{Duration};
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

    /// If the selected elements
    pub selected_sub_effect: Binding<Option<(ElementId, Arc<SubEffectDescription>)>>,

    /// The binding for the selected element (used when updating)
    selected_elements_binding: Binding<Arc<HashSet<ElementId>>>,

    /// The animation this model is for
    animation: Arc<Anim>,

    /// The layers in the current frame
    layers: BindRef<Vec<FrameLayerModel>>,

    /// The currently selected time
    current_time: BindRef<Duration>,

    /// The currently selected layer
    selected_layer: BindRef<Option<u64>>,
}

impl<Anim: Animation> Clone for SelectionModel<Anim> {
    fn clone(&self) -> Self {
        SelectionModel {
            selected_elements:          self.selected_elements.clone(),
            selection_in_order:         self.selection_in_order.clone(),
            selected_path:              self.selected_path.clone(),
            selected_elements_binding:  self.selected_elements_binding.clone(),
            selected_sub_effect:        self.selected_sub_effect.clone(),
            animation:                  self.animation.clone(),
            layers:                     self.layers.clone(),
            current_time:               self.current_time.clone(),
            selected_layer:             self.selected_layer.clone(),
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
        let selected_sub_effect         = bind(None);

        SelectionModel {
            selected_elements:          selected_elements,
            selected_elements_binding:  selected_elements_binding,
            selected_path:              selected_path,
            selection_in_order:         selection_in_order,
            selected_sub_effect:        selected_sub_effect,
            animation:                  animation,
            layers:                     frame_model.layers.clone(),
            current_time:               BindRef::from(&timeline_model.current_time),
            selected_layer:             BindRef::from(&timeline_model.selected_layer),
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
    /// Adds a particular element to the selection if it's not already present, or removes it if it is present
    ///
    pub fn toggle(&self, element: ElementId) {
        // Not *ideal* because there's a race condition here (between retrieving the hashset and writing it back again)
        let existing_selection = self.selected_elements_binding.get();

        let mut new_selection = (*existing_selection).clone();
        if new_selection.contains(&element) {
            new_selection.remove(&element);
        } else {
            new_selection.insert(element);
        }
        self.selected_elements_binding.set(Arc::new(new_selection));
    }

    ///
    /// Clears the current selection
    ///
    pub fn clear_selection(&self) {
        self.selected_elements_binding.set(Arc::new(HashSet::new()));
    }

    ///
    /// Returns true if the specified point is within the selection path
    ///
    pub fn point_in_selection_path(&self, x: f64, y: f64) -> bool {
        if let Some(path) = self.selected_path.get() {
            let subpaths = path.to_subpaths();
            subpaths.into_iter()
                .any(|subpath| path_contains_point(&subpath, &PathPoint { position: (x, y) }))
        } else {
            false
        }
    }
}

impl<Anim: 'static+EditableAnimation> SelectionModel<Anim> {
    ///
    /// Performs a cut operation using the current selection path on the specified layer, adding the elements inside
    /// the path to the current selection.
    ///
    /// Usually you should check that there are no selected elements and a path set before calling this.
    ///
    pub fn cut_selection(&self) {
        // Selection is cleared as a result of this operation
        self.selected_elements_binding.set(Arc::new(HashSet::new()));

        // Gather information required to make the cut action
        let path            = if let Some(path) = self.selected_path.get() { path } else { return; };
        let path            = Arc::new(path.elements().collect());
        let when            = self.current_time.get();
        let inside_group    = self.animation.assign_element_id();
        let layer_id        = self.selected_layer.get();
        let layer_id        = if let Some(layer_id) = layer_id { layer_id } else { return; };

        // Send the edits to the animation
        self.animation.perform_edits(vec![AnimationEdit::Layer(layer_id, LayerEdit::Cut {
            path, when, inside_group
        })]);

        // Read the contents of the inner group (cut has no effect if applied to a non-existent layer)
        let layers          = self.layers.get();
        let frame           = layers.iter().filter(|layer| layer.layer_id == layer_id).nth(0);
        let frame           = if let Some(frame) = frame { frame.frame.get() } else { return; };
        let frame           = if let Some(frame) = frame { frame } else { return; };

        let cut_elements    = frame.element_with_id(inside_group);
        let cut_elements    = if let Some(Vector::Group(cut_elements)) = &cut_elements { cut_elements.elements().collect::<Vec<_>>() } else { vec![] };

        // Ungroup the two cut groups
        self.animation.perform_edits(vec![AnimationEdit::Element(vec![inside_group], ElementEdit::Ungroup)]);

        // Set the selection to be the cut elements
        self.selected_elements_binding.set(Arc::new(cut_elements.into_iter().map(|elem| elem.id()).collect::<HashSet<_>>()));
    }
}
