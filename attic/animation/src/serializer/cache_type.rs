use super::source::*;
use super::target::*;
use super::super::traits::*;

impl CacheType {
    ///
    /// Generates a serialized version of these brush properties on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        match self {
            CacheType::OnionSkinLayer   => data.write_chr('O')
        }
    }

    ///
    /// Deserializes brush properties from a stream
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<CacheType> {
        match data.next_chr() {
            'O' => Some(CacheType::OnionSkinLayer),
            _   => None
        }
    }
}
