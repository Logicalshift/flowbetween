use super::resolve_element::*;
use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::sync::*;

impl Vector {
    ///
    /// Generates a serialized version of this vector element on the specified data target
    /// 
    /// Vector elements are serialized without their ID (this can be serialized separately if needed)
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::Vector::*;

        match self {
            Transformed(transform)      => { data.write_chr('T'); transform.serialize(data); }
            BrushDefinition(defn)       => { data.write_chr('D'); defn.serialize(data); }
            BrushProperties(props)      => { data.write_chr('P'); props.serialize(data); }
            BrushStroke(brush)          => { data.write_chr('s'); brush.serialize(data); }
            Path(path)                  => { data.write_chr('p'); path.serialize(data); }
            Motion(motion)              => { data.write_chr('m'); motion.serialize(data); }
            Group(group)                => { data.write_chr('g'); group.serialize(data); }
        }
    }

    ///
    /// Deserializes a vector element from a data source
    ///
    pub fn deserialize<Src: 'static+AnimationDataSource>(element_id: ElementId, data: &mut Src) -> Option<impl ResolveElements<Vector>> {
        // Function to turn a resolve function into a boxed resolve function (to get around limitations in Rust's type inference)
        fn box_fn<TFn: 'static+FnOnce(&dyn Fn(ElementId) -> Option<Vector>) -> Option<Vector>>(func: TFn) -> Box<dyn FnOnce(&dyn Fn(ElementId) -> Option<Vector>) -> Option<Vector>> {
            Box::new(func)
        }

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
            'p' => { unimplemented!("Path") }
            'm' => { unimplemented!("Motion") }
            'g' => { 
                GroupElement::deserialize(element_id, data)
                    .map(|group_resolver| box_fn(move |mapper| group_resolver.resolve(mapper)
                        .map(|group| Vector::Group(group))))
            }

            _ => None
        }?;

        // Return a resolver based on the deserialized data
        Some(ElementResolver(move |mapper| {
            (resolve)(mapper)
        }))
    }
}
