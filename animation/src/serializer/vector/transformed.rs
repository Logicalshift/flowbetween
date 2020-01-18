use super::super::target::*;
use super::super::super::traits::*;

impl TransformedVector {
    ///
    /// Generates a serialized version of this transformed vector on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        let original    = self.without_transformations();
        let transformed = self.transformed_vector();

        // Serialize the IDs of the transformed and original vectors
        original.id().serialize(data);
        transformed.id().serialize(data);

        // Only serialize the vectors themselves if they have no ID assigned
        if original.id().is_unassigned() {
            original.serialize(data);
        }

        if transformed.id().is_unassigned() {
            transformed.id().serialize(data);
        }
    }
}
