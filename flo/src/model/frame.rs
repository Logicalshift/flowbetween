use flo_stream::*;
use flo_binding::*;
use flo_animation::*;
use flo_curves::bezier::path::path_contains_point;

use futures::*;
use futures::future;
use futures::stream::{BoxStream};

use std::sync::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Duration;

///
/// Represents a match against a vector element
///
#[derive(Clone, Copy, PartialEq)]
pub enum ElementMatch {
    /// The point is inside the path for the specified element
    InsidePath(ElementId),

    /// The point is not inside the element path but is inside the element's bounding box
    OnlyInBounds(ElementId)
}

impl From<ElementMatch> for ElementId {
    fn from(item: ElementMatch) -> ElementId {
        match item {
            ElementMatch::InsidePath(val) => val,
            ElementMatch::OnlyInBounds(val) => val
        }
    }
}

///
/// Provides the model for a layer in the current frame
///
#[derive(Clone)]
pub struct FrameLayerModel {
    /// The ID of this layer
    pub layer_id: u64,

    /// The current frmae for this layer
    pub frame: BindRef<Option<Arc<dyn Frame>>>,
}

///
/// The frame model provides bindings for the content of the current frame
///
#[derive(Clone)]
pub struct FrameModel {
    /// Set to true if we should create a new keyframe when drawing (and there is no current keyframe)
    pub create_keyframe_on_draw: Binding<bool>,

    /// True if the current layer/selected time is on a keyframe
    pub keyframe_selected: BindRef<bool>,

    /// The previous and next keyframes
    pub previous_and_next_keyframe: BindRef<(Option<Duration>, Option<Duration>)>,

    /// The layers in the current frame
    pub layers: BindRef<Vec<FrameLayerModel>>,

    /// The currently selected frame (the current frame in the selected layer)
    pub frame: BindRef<Option<Arc<dyn Frame>>>,

    /// The elements in the current frame and their properties (all of the elements in the current frame in the selected layer)
    pub elements: BindRef<Arc<Vec<(Vector, Arc<VectorProperties>)>>>,

    /// The bounding boxes of all of the elements
    pub bounding_boxes: BindRef<Arc<HashMap<ElementId, Rect>>>
}

impl FrameModel {
    ///
    /// Creates a new frame model that tracks the specified animation
    ///
    /// The animation update binding can be updated whenever the frames become
    /// invalidated; the value has no meaning, so any value (for example, the
    /// length of the edit log)
    ///
    pub fn new<Anim: Animation+'static>(animation: Arc<Anim>, edits: Subscriber<Arc<Vec<AnimationEdit>>>, when: BindRef<Duration>, animation_update: BindRef<u64>, selected_layer: BindRef<Option<u64>>) -> FrameModel {
        // Create the bindings for the current frame state
        let keyframe_selected           = Self::keyframe_selected(Arc::clone(&animation), edits.resubscribe(), when.clone(), selected_layer.clone());
        let previous_and_next_keyframe  = Self::previous_next_keyframes(Arc::clone(&animation), edits.resubscribe(), when.clone(), selected_layer.clone());

        // The hashmap allows us to track frame bindings independently from layer bindings
        let frames: Mutex<HashMap<u64, FrameLayerModel>> = Mutex::new(HashMap::new());

        // Create a computed list of layers (because updates are lazy, this will
        // only update when it's actually read)
        let layers = computed(move || {
            // Claim the frames
            let mut frames = frames.lock().unwrap();

            // We bind to the update so this invalidates whenever the update list changes
            animation_update.get();

            // Refresh the frames from the animation
            let layer_ids = animation.get_layer_ids();

            // Remove layers that aren't in use any more
            let deleted_layers: Vec<_> = layer_ids
                .iter()
                .filter(|layer_id|  !frames.contains_key(layer_id))
                .map(|layer_id_ref| *layer_id_ref)
                .collect();

            deleted_layers.into_iter().for_each(|deleted_layer_id| { frames.remove(&deleted_layer_id); });

            // Update or generate the frame layer model (something bound to a single layer will get updates for that layer)
            for layer_id in layer_ids.iter() {
                match frames.entry(*layer_id) {
                    Entry::Occupied(_occupied) => (),

                    Entry::Vacant(vacant) => {
                        // Create a new bindnig
                        let layer_id            = *layer_id;
                        let when                = BindRef::clone(&when);
                        let frame_animation     = Arc::clone(&animation);
                        let animation_update    = animation_update.clone();

                        let frame_binding       = ComputedBinding::new_in_context(move || {
                            // Binds to the animation update...
                            animation_update.get();

                            // ... as well as the time
                            let when = when.get();

                            // Content is the frame from the layer at this time
                            frame_animation.get_layer_with_id(layer_id)
                                .map(|layer| layer.get_frame_at_time(when))
                        });

                        // Add a frame layer model for this frame
                        let frame           = BindRef::new(&frame_binding);

                        vacant.insert(FrameLayerModel {
                            layer_id:       layer_id,
                            frame:          frame,
                        });
                    }
                }
            }

            // Generate the final result
            layer_ids.into_iter()
                .map(|layer_id| frames.get(&layer_id).unwrap())
                .cloned()
                .collect()
        });

        // The current frame tracks the frame the user has got selected from the set of layers
        let frame                   = Self::current_frame(selected_layer, layers.clone());
        let elements                = Self::element_properties(frame.clone());
        let bounding_boxes          = Self::bounding_boxes(elements.clone());

        let create_keyframe_on_draw     = bind(true);

        // Result is a new FrameModel containing these layers
        FrameModel {
            create_keyframe_on_draw:    create_keyframe_on_draw,
            keyframe_selected:          keyframe_selected,
            previous_and_next_keyframe: previous_and_next_keyframe,
            layers:                     BindRef::new(&layers),
            frame:                      frame,
            elements:                   elements,
            bounding_boxes:             bounding_boxes
        }
    }

    ///
    /// Returns a binding for the selected frame
    ///
    fn current_frame<SelectedLayer: 'static+Bound<Option<u64>>, LayerModel: 'static+Bound<Vec<FrameLayerModel>>>(selected_layer: SelectedLayer, layers: LayerModel) -> BindRef<Option<Arc<dyn Frame>>> {
        BindRef::new(&computed(move || {
            let selected_layer_id = selected_layer.get();

            layers.get()
                .into_iter()
                .filter(|layer| Some(layer.layer_id) == selected_layer_id)
                .filter_map(|layer| layer.frame.get())
                .nth(0)
        }))
    }

    ///
    /// True if the animation edit affects the keyframes on the specified layer
    ///
    fn is_key_frame_update(layer_id: u64, edit: &AnimationEdit) -> bool {
        match edit {
            AnimationEdit::Layer(edit_layer_id, LayerEdit::AddKeyFrame(_)) |
            AnimationEdit::Layer(edit_layer_id, LayerEdit::RemoveKeyFrame(_)) => edit_layer_id == &layer_id,
            _ => false
        }
    }

    ///
    /// Stream of notifications that the current frame has updated
    ///
    fn frame_update_stream(edits: Subscriber<Arc<Vec<AnimationEdit>>>, when: BindRef<Duration>, selected_layer: BindRef<Option<u64>>) -> BoxStream<'static, ()> {
        // Events indicating a new key frame
        let selected_layer_2    = selected_layer.clone();
        let new_key_frame       = edits
            .filter_map(move |edits| {
                // Get the active layer
                let layer_id = selected_layer_2.get();

                if let Some(layer_id) = layer_id {
                    // Generate an event if the edits contain a Add or Remove for the current layer
                    if edits.iter().any(|edit| Self::is_key_frame_update(layer_id, edit)) {
                        future::ready(Some(()))
                    } else {
                        future::ready(None)
                    }
                } else {
                    // No events if there is no layer
                    future::ready(None)
                }
            });

        // Events indicating the selection has changed
        let when_changed            = follow(when).map(|_| ());
        let selected_layer_changed  = follow(selected_layer).map(|_| ());

        // If any of these events occur, then the keyframe may have changed
        Box::pin(stream::select(stream::select(new_key_frame, when_changed), selected_layer_changed))
    }

    ///
    /// Returns a binding indicating if a keyframe is currently selected
    ///
    fn keyframe_selected<Anim: Animation+'static>(animation: Arc<Anim>, edits: Subscriber<Arc<Vec<AnimationEdit>>>, when: BindRef<Duration>, selected_layer: BindRef<Option<u64>>) -> BindRef<bool> {
        // Get a stream of frame update events
        let frame_updates = Self::frame_update_stream(edits, when.clone(), selected_layer.clone());

        // Update the binding whenever they change
        let keyframe_selected = bind_stream(frame_updates, false, move |_, _| {
            // Get the current position in the timeline
            let when    = when.get();
            let layer   = selected_layer.get();

            if let Some(layer) = layer {
                // See if there's a keyframe at this exact time (well, within a millisecond)
                let layer       = animation.get_layer_with_id(layer);
                let one_ms      = Duration::from_millis(1);
                let start       = if when > one_ms { when - one_ms } else { Duration::from_millis(0) };
                let end         = when + one_ms;
                let keyframes   = layer.map(|layer| layer.get_key_frames_during_time(start..end).collect::<Vec<_>>());

                keyframes.map(|frames| frames.len() > 0).unwrap_or(false)
            } else {
                // No selected layer
                false
            }
        });

        BindRef::from(keyframe_selected)
    }

    ///
    /// Returns a binding of the previous and next keyframes
    ///
    fn previous_next_keyframes<Anim: Animation+'static>(animation: Arc<Anim>, edits: Subscriber<Arc<Vec<AnimationEdit>>>, when: BindRef<Duration>, selected_layer: BindRef<Option<u64>>) -> BindRef<(Option<Duration>, Option<Duration>)> {
        // Get a stream of frame update events
        let frame_updates = Self::frame_update_stream(edits, when.clone(), selected_layer.clone());

        // Update the binding whenever they change
        let previous_and_next = bind_stream(frame_updates, (None, None), move |_, _| {
            // Get the current position in the timeline
            let when    = when.get();
            let layer   = selected_layer.get();

            if let Some(layer) = layer {
                // See if there's a keyframe at this exact time (well, within a millisecond)
                let layer       = animation.get_layer_with_id(layer);
                let keyframes:Option<(Option<Duration>, Option<Duration>)>   = layer.map(|layer| layer.previous_and_next_key_frame(when));

                keyframes.unwrap_or((None, None))
            } else {
                // No selected layer
                (None, None)
            }
        });

        BindRef::from(previous_and_next)
    }
    ///
    /// Returns a binding mapping between the elements in a frame and their properties
    ///
    fn element_properties<CurrentFrame: 'static+Bound<Option<Arc<dyn Frame>>>>(current_frame: CurrentFrame) -> BindRef<Arc<Vec<(Vector, Arc<VectorProperties>)>>> {
        BindRef::new(&computed(move || {
            let mut result      = vec![];

            // Fetch the current frame
            let current_frame   = current_frame.get();

            if let Some(current_frame) = current_frame {
                // Get the elements for the current frame
                let elements                = current_frame.vector_elements();

                // current_properties will track the properties attached to each element
                // (TODO: in general we can generate properties individually for elements now)
                if let Some(elements) = elements {
                    for element in elements {
                        // Process how the properties change for this element
                        let current_properties = current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

                        // Add to the result
                        result.push((element, Arc::clone(&current_properties)));
                    }
                }
            }

            Arc::new(result)
        }))
    }

    ///
    /// Returns a binding that finds the bounding boxes of all of the vectors in the current frame
    ///
    fn bounding_boxes<Elements:'static+Bound<Arc<Vec<(Vector, Arc<VectorProperties>)>>>>(elements: Elements) -> BindRef<Arc<HashMap<ElementId, Rect>>> {
        BindRef::new(&computed(move || {
            let elements = elements.get();

            let bounding_boxes = elements.iter()
                .map(|(vector, properties)| {
                    let paths   = vector.to_path(properties, PathConversion::Fastest).unwrap_or_else(|| vec![]);
                    let bounds  = paths.into_iter().fold(Rect::empty(), |a, b| a.union(b.bounding_box()));

                    (vector.id(), bounds)
                });

            Arc::new(bounding_boxes.collect())
        }))
    }

    ///
    /// Returns the elements at the specified point
    ///
    pub fn elements_at_point(&self, point: (f32, f32)) -> impl Iterator<Item=ElementMatch> {
        // Fetch the elements and their bounding boxes
        let elements        = self.elements.get();
        let more_elements   = Arc::clone(&elements);
        let bounding_boxes  = self.bounding_boxes.get();

        let (x, y)          = point;
        let path_point      = PathPoint::new(x, y);

        // This would be considerably more elegant if rust understood that it could keep the Arc<Vec<_>> around to make the lifetime
        // of elements.iter() work out. We use array indexes and multiple references to the elements array instead here, so the elements
        // object can be owned by those functions.

        // Iterate through the elements in reverse
        let indexes = (0..elements.len()).into_iter().rev();

        // Filter to the elements where the point is inside the bounding box
        let inside_bounds = indexes.filter(move |element_index| {
            bounding_boxes.get(&elements[*element_index].0.id())
                .map(|bounds| bounds.contains(x, y))
                .unwrap_or(false)
        });

        // Generate a result based on whether or not the match is inside the path for the element
        let matches = inside_bounds
            .map(move |element_index| {
                // Get the vector properties from the more_elements array (elements is used above so we need two references)
                let &(ref vector, ref properties)   = &more_elements[element_index];
                let element_id                      = vector.id();

                // Convert the element to paths and check if the point is inside
                let paths                           = vector.to_path(properties, PathConversion::Fastest);
                let inside_path                     = paths.map(|paths| paths.into_iter().any(|path| path_contains_point(&path, &path_point))).unwrap_or(false);

                // Any match inside the bounds is a match, but we often treat a point inside the path as a stronger match
                if inside_path {
                    ElementMatch::InsidePath(element_id)
                } else {
                    ElementMatch::OnlyInBounds(element_id)
                }
            });

        matches
    }
}
