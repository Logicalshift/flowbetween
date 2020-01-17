use super::super::target::*;
use super::super::super::traits::*;

impl PathEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PathEdit::*;

        match self {
            CreatePath(elem, components)    => { data.write_chr('+'); elem.serialize(data); data.write_usize(components.len()); components.iter().for_each(|component| component.serialize(data)); },
            SelectBrush(elem, defn, style)  => { data.write_chr('S'); elem.serialize(data); defn.serialize(data); style.serialize(data); }
            BrushProperties(elem, props)    => { data.write_chr('P'); elem.serialize(data); props.serialize(data); }
        }
    }
}
