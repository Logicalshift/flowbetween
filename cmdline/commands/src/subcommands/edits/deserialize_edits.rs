use crate::state::*;
use crate::error::*;
use crate::output::*;

use flo_stream::*;
use flo_animation::*;

use futures::prelude::*;
use std::marker::{Unpin};

///
/// Deserializes edits read from an input stream of characters and adds them to the edit buffer
///
pub fn deserialize_edits<'a, SourceStream>(source: SourceStream, output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a 
where SourceStream: 'a+Unpin+Send+Stream<Item=char> {
    async move {
        let mut source      = source.fuse();
        let mut line_number = 1;
        let mut edits       = state.edit_buffer().clone();

        // Read from the stream until it is exhausted
        output.publish(FloCommandOutput::StartTask("Deserialize edits".to_string())).await;

        loop {
            match source.next().await {
                None        => { break; },
                Some('\n')  => { line_number += 1; }
                Some(' ')   |
                Some('\t')  |
                Some('\r')  => { /* Newlines and whitespace are ignored */ },

                Some(';')   => {
                    // Comment ended by a newline
                    loop {
                        match source.next().await {
                            Some('\n')  => { line_number += 1; break; }
                            Some('\r')  | 
                            None        => { break; },
                            _other      => { }
                        }
                    }
                }

                Some(other) => {
                    // Serialized edit, ended by a newline
                    let mut edit = String::new();
                    edit.push(other);

                    // Read until the newline to get the serialized edit
                    loop {
                        match source.next().await {
                            Some('\n')  => { break; }
                            None        => { break; },
                            Some(other) => { edit.push(other); }
                        }
                    }

                    // Attempt to deserialize the edit
                    let animation_edit = AnimationEdit::deserialize(&mut edit.chars());
                    let animation_edit = match animation_edit {
                        Some(animation_edit)    => Ok(animation_edit),
                        None                    => Err(CommandError::CannotParseEdit(line_number, edit))
                    }?;

                    // Add to the edit buffer
                    edits.push(animation_edit);

                    // Edits are ended by a newline, so the line number must increase
                    line_number += 1;
                }
            }
        }

        // Update the edit buffer with the values we just read
        *state = state.set_edit_buffer(edits);

        output.publish(FloCommandOutput::FinishTask).await;

        Ok(())
    }
}
