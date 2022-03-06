use super::element_id::*;
use super::layer_edit::*;
use super::motion_edit::*;
use super::element_edit::*;

use smallvec::*;
use std::iter;
use std::time::{Duration};

///
/// Represents an edit to an animation object
///
#[derive(Clone, PartialEq, Debug)]
pub enum AnimationEdit {
    /// Edit to an existing layer
    Layer(u64, LayerEdit),

    /// Edit to an existing element
    Element(Vec<ElementId>, ElementEdit),

    /// Edit to a motion (which is a description of how an element moves over time)
    /// Motions have element IDs so can be treated as elements but are not attached to a layer
    Motion(ElementId, MotionEdit),

    /// Sets the canvas size for this animation
    SetSize(f64, f64),

    /// Sets the lenght of a frame in this animation
    SetFrameLength(Duration),

    /// Sets the overall length of this animation
    SetLength(Duration),

    /// Adds a new layer and assigns it the specified ID
    /// Has no effect if a layer with that ID already exists
    AddNewLayer(u64),

    /// Removes the layer with the specified ID
    RemoveLayer(u64)
}

impl AnimationEdit {
    ///
    /// Retrieves the element IDs used by this edit
    ///
    pub fn used_element_ids(&self) -> SmallVec<[ElementId; 4]> {
        use AnimationEdit::*;

        match self {
            Layer(_, edit)                      => edit.used_element_ids(),
            Element(element_ids, element_edit)  => element_ids.iter().cloned().chain(element_edit.used_element_ids()).collect(),
            Motion(element_id, motion_edit)     => iter::once(*element_id).chain(motion_edit.used_element_ids()).collect(),

            SetSize(_, _)                       |
            SetFrameLength(_)                   |
            SetLength(_)                        |
            AddNewLayer(_)                      |
            RemoveLayer(_)                      => smallvec![]
        }
    }

    ///
    /// If this edit contains an unassigned element ID, calls the specified function to supply a new
    /// element ID. If the edit already has an ID, leaves it unchanged.
    ///
    pub fn assign_element_id<AssignFn: FnOnce() -> i64>(self, assign_element_id: AssignFn) -> AnimationEdit {
        use self::AnimationEdit::*;

        match self {
            Layer(layer_id, layer_edit) => Layer(layer_id, layer_edit.assign_element_id(assign_element_id)),
            other                       => other
        }
    }
}
