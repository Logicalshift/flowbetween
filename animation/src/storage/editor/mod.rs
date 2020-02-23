//!
//! The editor provides the EditableAnimation interface for anything that implements the storage API
//!

mod stream_animation;
mod stream_animation_core;
mod core_path;
mod core_paint;
mod core_layer;
mod core_motion;
mod core_element;
mod keyframe_core;
mod element_wrapper;

#[cfg(test)] mod tests;
#[cfg(test)] mod round_trip_tests;

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
