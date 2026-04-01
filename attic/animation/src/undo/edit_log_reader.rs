use crate::traits::*;

use futures::prelude::*;

use ::desync::*;
use flo_stream::*;

use std::sync::*;
use std::ops::{Range};

///
/// Reads the edit log for an animation
///
pub fn read_desync_edit_log<Anim>(animation: Arc<Desync<Anim>>, range: Range<usize>) -> impl Stream<Item=AnimationEdit>
where
    Anim: 'static + Unpin + Animation,
{
    const BLOCK_SIZE: usize = 50;

    // Read the edits from the source in BLOCK_SIZE chunks
    generator_stream(move |yield_value| {
        async move {
            let mut pos     = range.start;

            while pos < range.end {
                // Read a block of edits from the animation
                let end         = pos + BLOCK_SIZE;
                let end         = if end > range.end { range.end } else { end };
                let block_range = pos..end;

                let block       = animation.future_desync(|anim| {
                    async move {
                        anim.read_edit_log(block_range)
                            .collect::<Vec<_>>()
                            .await
                    }.boxed()
                }).await.ok().unwrap();

                // Return to the reader
                for item in block {
                    yield_value(item).await;
                }

                // Move on to the next block
                pos = end;
            }
        }
    })
}
