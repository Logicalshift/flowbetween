use super::resolve_element::*;
use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

impl GroupElement {
    ///
    /// Generates a serialized version of this group element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        use self::GroupType::*;
        match self.group_type() {
            Normal      => { data.write_chr('N'); }
            Added       => { data.write_chr('+'); }
        }

        // Grouped elements
        data.write_usize(self.num_elements());
        for elem in self.elements() {
            // Write out the ID of this elmeent
            elem.id().serialize(data);

            // Serialize the element if it has no ID (ie, not a reference to another element)
            // Elements with IDs are expected to be found elsewhere
            if elem.id().is_unassigned() {
                elem.serialize(data);
            }
        }
        
        // Hint path, if one is set
        if let Some(hint_path) = self.hint_path() {
            data.write_chr('H');
            data.write_usize(hint_path.len());
            hint_path.iter().for_each(|path| path.serialize(data));
        } else {
            data.write_chr('X');
        }
    }

    ///
    /// Deserializes a group from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<impl ResolveElements<GroupElement>> {
        match data.next_small_u64() {
            0 => {
                // Type of this group
                let group_type = match data.next_chr() {
                    'N'     => Some(GroupType::Normal),
                    '+'     => Some(GroupType::Added),
                    _       => None
                }?;

                // The elements
                enum Elem<LitType> {
                    Literal(LitType),
                    Reference(ElementId)
                }

                let num_elements = data.next_usize();
                let mut elements = vec![];

                for _ in 0..num_elements {
                    let element_id = ElementId::deserialize(data)?;

                    if element_id.is_unassigned() {
                        let element = Vector::deserialize(element_id, data)?;
                        elements.push(Elem::Literal(element));
                    } else {
                        elements.push(Elem::Reference(element_id));
                    }
                }

                // Hint path, if there is one
                let hint_path = if data.next_chr() == 'H' {
                    let hint_path_len   = data.next_usize();
                    let mut hint_path   = vec![];

                    for _ in 0..hint_path_len {
                        hint_path.push(Path::deserialize(data)?);
                    }

                    Some(Arc::new(hint_path))
                } else {
                    None
                };

                // Create a resolver for this group
                Some(ElementResolver(move |mapper| {
                    // Resolve the elements
                    let elements = elements.into_iter()
                        .map(|elem_ref| {
                            match elem_ref {
                                Elem::Literal(resolver)     => resolver.resolve(mapper).map(|vec| Arc::new(vec)),
                                Elem::Reference(element_id) => mapper(element_id)
                            }
                        })
                        .map(|elem| {
                            elem.map(|elem| (*elem).clone())
                        })
                        .collect::<Option<Vec<_>>>()?;

                    // Generate the final group element
                    let mut group = GroupElement::new(element_id, group_type, Arc::new(elements));
                    if let Some(hint_path) = hint_path {
                        group.set_hint_path(hint_path);
                    }
                    Some(group)
                }))
            }

            _ => None
        }
    }
}
