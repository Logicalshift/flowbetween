use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

impl AnimationEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::AnimationEdit::*;

        match self {
            Layer(layer_id, edit)       => { data.write_chr('L'); data.write_small_u64(*layer_id); edit.serialize(data); },
            Element(elements, edit)     => { data.write_chr('E'); data.write_usize(elements.len()); elements.iter().for_each(|elem| elem.serialize(data)); edit.serialize(data); },
            Motion(element, edit)       => { data.write_chr('M'); element.serialize(data); edit.serialize(data); },
            SetSize(width, height)      => { data.write_chr('S'); data.write_f64(*width); data.write_f64(*height); },
            AddNewLayer(layer_id)       => { data.write_chr('+'); data.write_small_u64(*layer_id); },
            RemoveLayer(layer_id)       => { data.write_chr('-'); data.write_small_u64(*layer_id); }
        }
    }

    ///
    /// Deserializes an animation edit
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<AnimationEdit> {
        match data.next_chr() {
            'L' => { let layer_id = data.next_small_u64(); LayerEdit::deserialize(data).map(move |edit| AnimationEdit::Layer(layer_id, edit)) }
            'E' => { unimplemented!("ElementEdit") }
            'M' => { unimplemented!("MotionEdit") }
            'S' => { Some(AnimationEdit::SetSize(data.next_f64(), data.next_f64())) }
            '+' => { Some(AnimationEdit::AddNewLayer(data.next_small_u64())) }
            '-' => { Some(AnimationEdit::RemoveLayer(data.next_small_u64())) }

            // Unknown character
            _   => None
        }
    }
}

///
/// Generates an animation as a serialized set of edits
///
pub fn serialize_animation_as_edits<'a, Tgt: AnimationDataTarget, Edits: IntoIterator<Item=&'a AnimationEdit>>(data: &mut Tgt, animation: Edits, title: &str) {
    // Write out a header including a readable version of the title
    data.write_chr('\n');

    data.write_chr(';');
    data.write_chr(' ');
    data.write_chr('-');
    data.write_chr(' ');

    for c in title.chars() {
        if c != '\n' {
            data.write_chr(c);
        }
    }

    data.write_chr(' ');
    data.write_chr('-');
    data.write_chr(' ');
    data.write_chr('\n');

    data.write_chr('\n');

    // Write out the actual animation edits
    animation.into_iter().for_each(|edit| {
        edit.serialize(data);
        data.write_chr('\n');
    });

    data.write_chr('\n');

    // Simple footer to finish the file
    data.write_chr(';');
    data.write_chr(' ');
    data.write_chr('-');
    data.write_chr('-');
    data.write_chr(' ');
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_size() {
        let mut encoded = String::new();
        AnimationEdit::SetSize(1024.0, 768.0).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::SetSize(1024.0, 768.0)));
    }

    #[test]
    fn add_new_layer() {
        let mut encoded = String::new();
        AnimationEdit::AddNewLayer(1).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::AddNewLayer(1)));
    }

    #[test]
    fn remove_layer() {
        let mut encoded = String::new();
        AnimationEdit::RemoveLayer(42).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::RemoveLayer(42)));
    }
}
