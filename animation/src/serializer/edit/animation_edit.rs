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
