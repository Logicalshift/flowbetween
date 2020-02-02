//!
//! The editor provides the EditableAnimation interface for anything that implements the storage API
//!

mod stream_animation;

use self::stream_animation::*;
use super::storage_api::*;
use super::super::traits::*;

use futures::stream::{BoxStream};

///
/// Creates an editable animation for a stream from the storage layer.
/// 
/// The output from the storage layer is passed in as a stream. This returns a stream that should be used
/// as the input. The output stream should initially block, and should post one value for every value
/// posted on the input stream that this function returns.
///
pub fn create_animation_editor<ConnectStream: FnOnce(BoxStream<'static, Vec<StorageCommand>>) -> BoxStream<'static, Vec<StorageResponse>>>(connect_stream: ConnectStream) -> impl EditableAnimation {
    StreamAnimation::new(connect_stream)
}
