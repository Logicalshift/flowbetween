//!
//! The editor provides the EditableAnimation interface for anything that implements the storage API
//!

mod stream_animation;

use self::stream_animation::*;
use super::storage_api::*;
use super::super::traits::*;

use futures::*;

///
/// Creates an editable animation for a stream from the storage layer.
/// 
/// The output from the storage layer is passed in as a stream. This returns a stream that should be used
/// as the input. The output stream should initially block, and should post one value for every value
/// posted on the input stream that this function returns.
///
pub fn create_animation_editor<TOutputStream: 'static+Send+Unpin+Stream<Item=Vec<StorageResponse>>>(output_stream: TOutputStream) -> (impl EditableAnimation, impl Stream<Item=Vec<StorageCommand>>+Unpin) {
    StreamAnimation::new(output_stream)
}
