use super::timeline::*;
use super::super::style::*;

use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::*;

use std::sync::*;
use std::time::Duration;
use std::marker::PhantomData;
use std::collections::HashMap;

///
/// Onion skin time, indicating whether or not it's before or after the current frame 
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OnionSkinTime {
    /// An onion skin displayed before the current frame
    BeforeFrame(Duration),

    /// An onion skin displayed after the current frame
    AfterFrame(Duration)
}

impl Into<Duration> for OnionSkinTime {
    fn into(self) -> Duration {
        match self {
            OnionSkinTime::BeforeFrame(when)    => when,
            OnionSkinTime::AfterFrame(when)     => when
        }
    }
}

///
/// The model used to describe which onion skins are being displayed
///
pub struct OnionSkinModel<Anim: Animation> {
    /// The colour of the future onion skins
    pub future_color: Binding<Color>,

    /// The colour of the past onion skins
    pub past_color: Binding<Color>,

    /// Whether or not the onion skins should be displayed
    pub show_onion_skins: Binding<bool>,

    /// The drawing actions for the onion skins to display (ordered as for onion_skin_times)
    pub onion_skins: BindRef<Vec<(OnionSkinTime, Vec<Draw>)>>,

    /// The times of the onion skins to display (ordered from most recent onion skin to least recent)
    pub onion_skin_times: BindRef<Vec<OnionSkinTime>>,

    /// The number of frames to show before the current frame
    pub frames_before: Binding<usize>,

    /// The number of frames to show after the current frame
    pub frames_after: Binding<usize>,

    anim: PhantomData<Anim>
}

impl<Anim: 'static+Animation> OnionSkinModel<Anim> {
    ///
    /// Creates a new onion skin model
    ///
    pub fn new(animation: Arc<Anim>, timeline: &TimelineModel<Anim>) -> OnionSkinModel<Anim> {
        // Create the basic bindings
        let future_color        = Binding::new(ONIONSKIN_FUTURE);
        let past_color          = Binding::new(ONIONSKIN_PAST);
        let show_onion_skins    = Binding::new(false);
        let frames_before       = Binding::new(3);
        let frames_after        = Binding::new(3);

        // Create the derived bindings
        let onion_skin_times    = Self::onion_skin_times(timeline, BindRef::from(&show_onion_skins), BindRef::from(&frames_before), BindRef::from(&frames_after));
        let onion_skins         = Self::onion_skins(Arc::clone(&animation), BindRef::from(&timeline.selected_layer), BindRef::clone(&onion_skin_times));

        OnionSkinModel {
            future_color:       future_color,
            past_color:         past_color,
            show_onion_skins:   show_onion_skins,
            frames_before:      frames_before,
            frames_after:       frames_after,
            onion_skin_times:   onion_skin_times,
            onion_skins:        onion_skins,
            anim:               PhantomData
        }
    }

    ///
    /// Returns the current set of times to display onion skins for
    ///
    fn onion_skin_times(timeline: &TimelineModel<Anim>, show_onion_skins: BindRef<bool>, frames_before: BindRef<usize>, frames_after: BindRef<usize>) -> BindRef<Vec<OnionSkinTime>> {
        // Fetch the timeline properties
        let current_time        = timeline.current_time.clone();
        let frame_duration      = timeline.frame_duration.clone();
        let timeline_duration   = timeline.duration.clone();

        // Create the onion skin times computed binding
        let times = computed(move || {
            let show_onion_skins = show_onion_skins.get();

            if show_onion_skins {
                // Displaying the onion skins, so calculate the times we need to fetch
                let current_time        = current_time.get();
                let frame_duration      = frame_duration.get();
                let timeline_duration   = timeline_duration.get();
                let frames_before       = frames_before.get();
                let frames_after        = frames_after.get();

                // The maximum number of frames we want to calculate
                let max_frames          = frames_before.max(frames_after);

                // Compute frames starting from the current time in both directions (so frames are ordered in terms of distance from the current time, with most recent frames first)
                let mut onion_skin_times = vec![];
                for frame_num in 1..=max_frames {
                    // Time offset for this frame
                    let offset = frame_duration * (frame_num as u32);

                    // Past frame
                    if frame_num <= frames_before && offset <= current_time {
                        onion_skin_times.push(OnionSkinTime::BeforeFrame(current_time - offset));
                    }

                    // Future frame
                    if frame_num <= frames_after && current_time + offset <= timeline_duration {
                        onion_skin_times.push(OnionSkinTime::AfterFrame(current_time + offset));
                    }
                }

                onion_skin_times
            } else {
                // Not showing any onion skins, so there are no times to display
                vec![]
            }
        });

        BindRef::from(times)
    }

    ///
    /// Returns a binding for the set of drawing actions to draw the current set of onion skins
    ///
    fn onion_skins(animation: Arc<Anim>, selected_layer: BindRef<Option<u64>>, onion_skin_times: BindRef<Vec<OnionSkinTime>>) -> BindRef<Vec<(OnionSkinTime, Vec<Draw>)>> {
        // Stream of onion skin updates, combined with the currently selected layer
        let onion_skin_times    = computed(move || (selected_layer.get(), onion_skin_times.get()));
        let onion_skin_stream   = follow(onion_skin_times);

        // Binding created from following the onion skin stream and attempting to fetch the cached onion skins
        let fetching_onion_skins = onion_skin_stream.map(move |(selected_layer, onion_skin_times)| {
            let animation = Arc::clone(&animation);
            selected_layer.map(move |selected_layer| {
                // Fetch the layer
                let layer       = animation.get_layer_with_id(selected_layer);

                // Generate the list of cached values for the onion skins
                let mut fetch   = vec![];

                for time in onion_skin_times.into_iter() {
                    let when: Duration  = time.into();
                    let cache           = layer.map(|layer| layer.get_canvas_cache_at_time(when));
                    let onion_skin      = cache.map(|cache| cache.retrieve_or_generate(CacheType::OnionSkinLayer, Box::new(|| vec![])));
                    let onion_skin      = onion_skin.unwrap_or(CacheProcess::Cached(vec![]));

                    fetch.push((time, onion_skin));
                }

                fetch
            }).unwrap_or(vec![])
        });

        BindRef::from(Binding::new(vec![]))
    }
}

impl<Anim: Animation> Clone for OnionSkinModel<Anim> {
    fn clone(&self) -> OnionSkinModel<Anim> {
        OnionSkinModel {
            future_color:       self.future_color.clone(),
            past_color:         self.past_color.clone(),
            show_onion_skins:   self.show_onion_skins.clone(),
            frames_before:      self.frames_before.clone(),
            frames_after:       self.frames_after.clone(),
            onion_skin_times:   self.onion_skin_times.clone(),
            onion_skins:        self.onion_skins.clone(),
            anim:               PhantomData
        }
    }
}
