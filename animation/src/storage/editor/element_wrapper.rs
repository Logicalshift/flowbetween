use super::super::super::traits::*;
use super::super::super::serializer::*;

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

    /// The element that this is ordered before (none = ordered by element ID)
    pub order_before: Option<ElementId>,

    /// The element that this is ordered after (none = ordered by element ID)
    pub order_after: Option<ElementId>
}

impl ElementWrapper {
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

        // Order before/after
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
                        order_before,
                        order_after
                    })
                }))
            }

            _ => None
        }
    }
}
