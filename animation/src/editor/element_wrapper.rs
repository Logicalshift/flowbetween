use crate::traits::*;
use crate::serializer::*;

use std::str::{Chars};
use std::time::{Duration};

///
/// Serializable wrapper for elements
///
#[derive(Clone)]
pub struct ElementWrapper {
    /// The serialized element
    pub element: Vector,

    /// When this element should appear
    pub start_time: Duration,

    /// The elements that are attached to this element
    pub attachments: Vec<ElementId>,

    /// The elements that this is attached to
    pub attached_to: Vec<ElementId>,

    /// True if this element is used by the keyframe it's attached to but is not part of the render path
    pub unattached: bool,

    /// The 'parent' element for this one (eg, if it's part of a group)
    pub parent: Option<ElementId>,

    /// The element that this is ordered before (none = ordered by element ID)
    pub order_before: Option<ElementId>,

    /// The element that this is ordered after (none = ordered by element ID)
    pub order_after: Option<ElementId>
}

impl ElementWrapper {
    ///
    /// Creates an element wrapper containing a particular element with its other properties sert to the default values
    ///
    pub fn attached_with_element(element: Vector, when: Duration) -> ElementWrapper {
        Self::with_element(element, when, false)
    }

    ///
    /// Creates an element wrapper containing a particular element with its other properties sert to the default values
    ///
    pub fn unattached_with_element(element: Vector, when: Duration) -> ElementWrapper {
        Self::with_element(element, when, true)
    }

    ///
    /// Creates an element wrapper containing a particular element with its other properties sert to the default values
    ///
    fn with_element(element: Vector, when: Duration, unattached: bool) -> ElementWrapper {
        ElementWrapper {
            element:        element,
            start_time:     when,
            attachments:    vec![],
            attached_to:    vec![],
            unattached:     unattached,
            parent:         None,
            order_before:   None,
            order_after:    None
        }
    }

    ///
    /// Creates an 'error' element wrapper
    ///
    pub fn error() -> ElementWrapper {
        ElementWrapper {
            element:        Vector::Error,
            start_time:     Duration::from_micros(0),
            attachments:    vec![],
            attached_to:    vec![],
            unattached:     false,
            parent:         None,
            order_before:   None,
            order_after:    None
        }
    }

    ///
    /// Generates a string version of this element
    ///
    pub fn serialize_to_string(&self) -> String {
        let mut result = String::new();
        self.serialize(&mut result);
        result
    }

    ///
    /// Writes this element to a data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        // The element itself
        self.element.serialize(data);

        data.write_duration(self.start_time);

        // Attachments, if any
        data.write_usize(self.attachments.len());
        self.attachments.iter().for_each(|attachment| attachment.serialize(data));

        data.write_usize(self.attached_to.len());
        self.attached_to.iter().for_each(|attached_to| attached_to.serialize(data));

        if self.unattached {
            data.write_chr('U');
        } else {
            data.write_chr('A');
        }

        // Parent, and order before/after
        if let Some(parent) = self.parent.as_ref() {
            data.write_chr('+');
            parent.serialize(data);
        } else {
            data.write_chr('-');
        }

        if let Some(order_before) = self.order_before.as_ref() {
            data.write_chr('+');
            order_before.serialize(data);
        } else {
            data.write_chr('-');
        }

        if let Some(order_after) = self.order_after.as_ref() {
            data.write_chr('+');
            order_after.serialize(data);
        } else {
            data.write_chr('-');
        }
    }

    ///
    /// Deserializes from a data source
    ///
    pub fn deserialize(element_id: ElementId, data: &mut Chars) -> Option<impl ResolveElements<ElementWrapper>> {
        match data.next_small_u64() {
            0 => {
                // Version 0
                let element         = Vector::deserialize(element_id, data)?;
                let start_time      = data.next_duration();

                let num_attachments = data.next_usize();
                let attachments     = (0..num_attachments).into_iter()
                    .map(|_| ElementId::deserialize(data))
                    .collect::<Option<Vec<_>>>()?;

                let num_attached_to = data.next_usize();
                let attached_to     = (0..num_attached_to).into_iter()
                    .map(|_| ElementId::deserialize(data))
                    .collect::<Option<Vec<_>>>()?;

                let unattached      = data.next_chr() == 'U';

                let parent          = match data.next_chr() {
                    '+' => { ElementId::deserialize(data).map(|id| Some(id)) }
                    '-' => { Some(None) }
                    _   => None
                }?;

                let order_before    = match data.next_chr() {
                    '+' => { ElementId::deserialize(data).map(|id| Some(id)) }
                    '-' => { Some(None) }
                    _   => None
                }?;

                let order_after     = match data.next_chr() {
                    '+' => { ElementId::deserialize(data).map(|id| Some(id)) }
                    '-' => { Some(None) }
                    _   => None
                }?;

                // Result is a resolver that creates the wrapper
                Some(ElementResolver(move |mapper| {
                    let element = element.resolve(mapper)?;

                    Some(ElementWrapper {
                        element,
                        start_time,
                        attachments,
                        attached_to,
                        unattached,
                        parent,
                        order_before,
                        order_after
                    })
                }))
            }

            _ => None
        }
    }
}
