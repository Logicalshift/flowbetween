use super::super::target::*;
use super::super::super::traits::*;

impl Vector {
    ///
    /// Generates a serialized version of this vector element on the specified data target
    /// 
    /// Vector elements are serialized without their ID (this can be serialized separately if needed)
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::Vector::*;

        match self {
            Transformed(transformed)    => { data.write_chr('T'); unimplemented!() }
            BrushDefinition(defn)       => { data.write_chr('D'); defn.serialize(data); }
            BrushProperties(props)      => { data.write_chr('P'); props.serialize(data); }
            BrushStroke(brush)          => { data.write_chr('s'); brush.serialize(data); }
            Path(path)                  => { data.write_chr('p'); unimplemented!() }
            Motion(motion)              => { data.write_chr('m'); unimplemented!() }
            Group(group)                => { data.write_chr('g'); unimplemented!() }
        }
    }
}
