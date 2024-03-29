use super::resolve_element::*;
use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use smallvec::*;

use std::str::{Chars};

// Function to turn a resolve function into a boxed resolve function (to get around limitations in Rust's type inference)
fn box_fn<TFn: 'static+FnOnce(&mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<Vector>>(func: TFn) -> Box<dyn FnOnce(&mut dyn FnMut(ElementId) -> Option<Vector>) -> Option<Vector>> {
    Box::new(func)
}

impl Vector {
    ///
    /// Returns true if the deserialization for this vector will 
    ///
    pub fn requires_resolution_for_deserialize(&self) -> bool {
        use self::Vector::*;

        match self {
            // Vectors that directly contain other elements will need those elements to be resolved (which can't be done when deserializing an edit log)
            Transformed(_transform)             => { true }
            Motion(_motion)                     => { true }
            Group(_group)                       => { true }
            Error                               => { true }

            BrushDefinition(_defn)              => { false }
            BrushProperties(_props)             => { false }
            BrushStroke(_brush)                 => { false }
            Path(_path)                         => { false }
            Shape(_shape)                       => { false }
            AnimationRegion(_region)            => { false }
            Transformation((_id, _transform))   => { false }
        }
    }

    ///
    /// Generates a serialized version of this vector element on the specified data target
    /// 
    /// Vector elements are serialized without their ID (this can be serialized separately if needed)
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::Vector::*;

        match self {
            Transformed(transform)          => { data.write_chr('T'); transform.serialize(data); }
            BrushDefinition(defn)           => { data.write_chr('D'); defn.serialize(data); }
            BrushProperties(props)          => { data.write_chr('P'); props.serialize(data); }
            BrushStroke(brush)              => { data.write_chr('s'); brush.serialize(data); }
            Path(path)                      => { data.write_chr('p'); path.serialize(data); }
            Shape(shape)                    => { data.write_chr('S'); shape.serialize(data); }
            Motion(motion)                  => { data.write_chr('m'); motion.serialize(data); }
            Group(group)                    => { data.write_chr('g'); group.serialize(data); }
            AnimationRegion(region)         => { data.write_chr('A'); region.serialize(data); }
            Error                           => { data.write_chr('?'); }

            Transformation((id, transform)) => { 
                data.write_chr('t'); 
                id.serialize(data); 

                data.write_usize(transform.len());
                transform.iter().for_each(|item| item.serialize(data));
            }
        }
    }

    ///
    /// Deserializes a vector element from a data source
    /// 
    /// We use a concrete type 'Chars' for the data source here and can't use generic types using the AnimationDataSource
    /// trait: this is because Rust's lifetime checker seems to have a bug and borrows the `data` element for as long
    /// as the trait exists if we use the generic form due to some kind of interaction with the box_fn function
    /// (data is only required as long as say `GroupElement::deserialize()` is running but Rust can't see that?)
    ///
    pub fn deserialize(element_id: ElementId, data: &mut Chars) -> Option<impl ResolveElements<Vector>> {
        // Deserialize the element
        let resolve = match data.next_chr() {
            'T' => { unimplemented!("Transformed") }
            'D' => { 
                BrushDefinitionElement::deserialize(element_id, data)
                    .map(|defn| box_fn(move |_| Some(Vector::BrushDefinition(defn))))
            }
            'P' =>  { 
                BrushPropertiesElement::deserialize(element_id, data)
                    .map(|properties| box_fn(move |_| Some(Vector::BrushProperties(properties))))
            }
            's' => {
                BrushElement::deserialize(element_id, data)
                    .map(|brush_stroke| box_fn(move |_| Some(Vector::BrushStroke(brush_stroke))))
            }
            'p' => {
                let (path, _attachments) = PathElement::deserialize(element_id, data)?;

                Some(box_fn(move |_| {
                    Some(Vector::Path(path))
                }))
            }
            'S' => {
                let shape = ShapeElement::deserialize(element_id, data)?;
                Some(box_fn(move |_| { 
                    Some(Vector::Shape(shape))
                }))
            }
            'A' => {
                let animation_element = AnimationElement::deserialize(element_id, data)?;
                Some(box_fn(move |_| { 
                    Some(Vector::AnimationRegion(animation_element))
                }))
            }
            'm' => { 
                MotionElement::deserialize(element_id, data)
                    .map(|motion| box_fn(move |_| Some(Vector::Motion(motion))))
            }
            'g' => { 
                let group_resolver = GroupElement::deserialize(element_id, data)?;
                Some(box_fn(move |mapper| {
                    let group = group_resolver.resolve(mapper)?;
                    Some(Vector::Group(group))
                }))
            }
            't' => {
                ElementId::deserialize(data)
                    .and_then(|elem_id| {
                        let num_items   = data.next_usize();
                        let transforms  = (0..num_items).into_iter()
                            .map(|_| Transformation::deserialize(data))
                            .collect::<Option<SmallVec<[_; 2]>>>();

                        transforms.map(move |transforms| (elem_id, transforms))
                    })
                    .map(|transform| Vector::Transformation(transform))
                    .map(|vector| box_fn(move |_| Some(vector)))
            }
            '?' => {
                Some(box_fn(move |_mapper| {
                    Some(Vector::Error)
                }))
            }

            _ => None
        }?;

        // Return a resolver based on the deserialized data
        Some(ElementResolver(move |mapper| {
            (resolve)(mapper)
        }))
    }
}
