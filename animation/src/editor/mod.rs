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
mod keyframe_raycast;
mod pending_storage_change;
mod paint_fill;
mod element_wrapper;
mod element_collide;
mod element_transform;
mod element_convert_to_path;
mod stream_layer;
mod stream_frame;
mod stream_layer_cache;

#[cfg(test)] mod tests;

use self::stream_animation::*;
use crate::storage::storage_api::*;
use crate::traits::*;

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
