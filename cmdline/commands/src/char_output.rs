use super::output::*;

use futures::prelude::*;

///
/// Converts a stream of command outputs into characters for display on a terminal
///
pub fn to_char_output<InputStream>(input: InputStream, format_width: u32) -> impl Stream<Item=char>+Send+Unpin
where InputStream: Stream<Item=FloCommandOutput>+Send+Unpin {
    input
        .map(|output| {
            use self::FloCommandOutput::*;

            match output {
                BeginCommand(_cmd)      => stream::iter(vec![]).boxed(),
                Message(msg)            => stream::iter((msg + "\n").chars().collect::<Vec<_>>()).boxed(),
                Error(err)              => stream::iter((err + "\n").chars().collect::<Vec<_>>()).boxed(),
                FinishCommand(_cmd)     => stream::iter(vec![]).boxed(),
                State(_new_state)       => stream::iter(vec![]).boxed(),
                Failure(error)          => stream::iter(format!("\nERROR: {}\n", error).chars().collect::<Vec<_>>()).boxed()
            }
        })
        .flatten()
}
