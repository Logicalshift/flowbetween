use super::super::source::*;
use super::super::target::*;
use crate::traits::*;

use std::str::{Chars};
use std::time::{Duration};

impl AnimationEdit {
    ///
    /// Returns true if this edit should be written to the log
    ///
    /// Some edits (such as those relating to an in-progress undo action) are not serialized to the edit log
    ///
    pub fn is_serialized(&self) -> bool {
        use self::AnimationEdit::*;
        use self::UndoEdit::*;

        match self {
            Layer(_, _)             |
            Element(_, _)           |
            Motion(_, _)            |
            SetSize(_, _)           |
            SetFrameLength(_)       |
            SetLength(_)            |
            AddNewLayer(_)          |
            RemoveLayer(_)          => true,

            Undo(FinishAction)      => true,

            Undo(PrepareToUndo(_))  |
            Undo(CompletedUndo(_))  |
            Undo(FailedUndo(_))     |
            Undo(PerformUndo { original_actions: _, undo_actions: _ }) => false,
        }
    }

    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::AnimationEdit::*;
        use self::UndoEdit::*;

        match self {
            Layer(layer_id, edit)       => { data.write_chr('L'); data.write_small_u64(*layer_id); edit.serialize(data); },
            Element(elements, edit)     => { data.write_chr('E'); data.write_usize(elements.len()); elements.iter().for_each(|elem| elem.serialize(data)); edit.serialize(data); },
            Motion(element, edit)       => { data.write_chr('M'); element.serialize(data); edit.serialize(data); },
            SetSize(width, height)      => { data.write_chr('S'); data.write_f64(*width); data.write_f64(*height); },
            SetFrameLength(len)         => { data.write_chr('f'); data.write_small_u64(len.as_nanos() as _); }
            SetLength(len)              => { data.write_chr('l'); data.write_duration(*len); },
            AddNewLayer(layer_id)       => { data.write_chr('+'); data.write_small_u64(*layer_id); },
            RemoveLayer(layer_id)       => { data.write_chr('-'); data.write_small_u64(*layer_id); },

            Undo(FinishAction)          => { data.write_chr('U'); data.write_chr('-'); },

            // Most undo actions are not actually serialized (they're used for internal signalling instead)
            Undo(PrepareToUndo(_))      => { },
            Undo(CompletedUndo(_))      => { },
            Undo(FailedUndo(_))         => { },
            Undo(PerformUndo { original_actions: _, undo_actions: _ }) => { },
        }
    }

    ///
    /// Deserializes an animation edit
    ///
    pub fn deserialize(data: &mut Chars) -> Option<AnimationEdit> {
        match data.next_chr() {
            'L' => { let layer_id = data.next_small_u64(); LayerEdit::deserialize(data).map(move |edit| AnimationEdit::Layer(layer_id, edit)) }
            'M' => { ElementId::deserialize(data).and_then(|elem| MotionEdit::deserialize(data).map(move |edit| AnimationEdit::Motion(elem, edit))) }
            'S' => { Some(AnimationEdit::SetSize(data.next_f64(), data.next_f64())) }
            'f' => { Some(AnimationEdit::SetFrameLength(Duration::from_nanos(data.next_small_u64() as _))) }
            'l' => { Some(AnimationEdit::SetLength(data.next_duration())) }
            '+' => { Some(AnimationEdit::AddNewLayer(data.next_small_u64())) }
            '-' => { Some(AnimationEdit::RemoveLayer(data.next_small_u64())) }

            'E' => { 
                let num_elements    = data.next_usize();
                let elements        = (0..num_elements).into_iter()
                    .map(|_| ElementId::deserialize(data))
                    .collect::<Option<Vec<_>>>();

                elements.and_then(|elements|
                    ElementEdit::deserialize(data)
                        .map(move |edit| AnimationEdit::Element(elements, edit)))
            }

            'U' => {
                match data.next_chr() {
                    '-'     => Some(AnimationEdit::Undo(UndoEdit::FinishAction)),
                    _       => None,
                }
            }

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
    use std::time::{Duration};

    #[test]
    fn set_size() {
        let mut encoded = String::new();
        AnimationEdit::SetSize(1024.0, 768.0).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::SetSize(1024.0, 768.0)));
    }

    #[test]
    fn set_frame_length() {
        let mut encoded = String::new();
        AnimationEdit::SetFrameLength(Duration::from_nanos(1_000_000_000 / 24)).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::SetFrameLength(Duration::from_nanos(1_000_000_000 / 24))));
    }

    #[test]
    fn set_length() {
        let mut encoded = String::new();
        AnimationEdit::SetLength(Duration::from_secs(80)).serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(AnimationEdit::SetLength(Duration::from_secs(80))));
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

    #[test]
    fn layer_edit() {
        let mut encoded = String::new();
        let edit        = AnimationEdit::Layer(1, LayerEdit::AddKeyFrame(Duration::from_millis(1000)));
        edit.serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn element_edit() {
        let mut encoded = String::new();
        let edit        = AnimationEdit::Element(vec![ElementId::Assigned(42), ElementId::Assigned(43), ElementId::Assigned(44)], ElementEdit::Delete);
        edit.serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn motion_edit() {
        let mut encoded = String::new();
        let edit        = AnimationEdit::Motion(ElementId::Assigned(42), MotionEdit::Create);
        edit.serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }

    #[test]
    fn undo_finish_action() {
        let mut encoded = String::new();
        let edit        = AnimationEdit::Undo(UndoEdit::FinishAction);
        edit.serialize(&mut encoded);

        assert!(AnimationEdit::deserialize(&mut encoded.chars()) == Some(edit));
    }
}
