use super::super::target::*;
use super::super::super::traits::*;

impl PathElement {
    ///
    /// Generates a serialized version of this path element on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // v0
        data.write_small_u64(0);

        // Write out the IDs of the property elements
        self.brush().id().serialize(data);
        self.properties().id().serialize(data);

        // If the IDs are unassigned then include teh properties/brush directly
        if self.brush().id().is_unassigned() {
            self.brush().serialize(data);
        }
        if self.properties().id().is_unassigned() {
            self.properties().serialize(data);
        }

        // Write out the path components
        self.path().serialize(data);
    }
}

impl Path {
    ///
    /// Generates a serialized version of this path on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        // Write out the path components
        let components = &self.elements;
        data.write_usize(components.len()); 

        let mut last_point = PathPoint::new(0.0, 0.0);
        for component in components.iter() {
            last_point = component.serialize_next(&last_point, data);
        }
    }
}