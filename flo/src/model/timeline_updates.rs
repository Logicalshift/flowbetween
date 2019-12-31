use flo_animation::*;

use futures::*;
use futures::stream;

use std::sync::*;
use std::time::Duration;

///
/// Model update actions for the timeline model
///
pub enum TimelineModelUpdate {
    AddNewLayer(u64),
    RemoveLayer(u64),
    AddKeyFrame(u64, Duration),
    RemoveKeyFrame(u64, Duration)
}

impl TimelineModelUpdate {
    ///
    /// Returns true if this operation is one affecting layers as a whole (eg, adding or removing a layer)
    ///
    pub fn is_layer_operation(&self) -> bool {
        use self::TimelineModelUpdate::*;

        match self {
            AddNewLayer(_)  |
            RemoveLayer(_)  =>  true,

            _               => false
        }
    }
}

///
/// Converts a stream of animation edits to a stream of timeline model updates
///
pub fn get_timeline_updates<EditStream: Stream<Item=Arc<Vec<AnimationEdit>>>>(edit_stream: EditStream) -> impl Stream<Item=TimelineModelUpdate> {
    edit_stream
        .map(|animation_edits| {
            use self::LayerEdit::*;
            use self::AnimationEdit::*;

            animation_edits.iter()
                .filter_map(|animation_edit| {
                    match animation_edit {
                        AddNewLayer(layer_id)                   => Some(TimelineModelUpdate::AddNewLayer(*layer_id)),
                        RemoveLayer(layer_id)                   => Some(TimelineModelUpdate::RemoveLayer(*layer_id)),
                        Layer(layer_id, AddKeyFrame(when))      => Some(TimelineModelUpdate::AddKeyFrame(*layer_id, *when)),
                        Layer(layer_id, RemoveKeyFrame(when))   => Some(TimelineModelUpdate::RemoveKeyFrame(*layer_id, *when)),

                        _                                       => None
                    }
                })
                .collect::<Vec<_>>()
        })
        .map(|edit_vec| stream::iter(edit_vec))
        .flatten()
}
