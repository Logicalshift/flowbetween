use crate::region::*;

use std::sync::*;
use std::time::{Duration};

///
/// Animation effect that animates a region frame-by-frame
///
/// Normally, new frames add their drawing to the region: this only shows the drawing added at the most recent moment in time
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameByFrameEffect {
    /// The whole content of the region is replaced by the new frame
    ReplaceWhole,

    /// The content at time 0 is always added to
    AddToInitial
}

impl FrameByFrameEffect {
    ///
    /// Animates a frame-by-frame effect
    ///
    pub fn animate_effect(effect: FrameByFrameEffect, content: &Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        // Collect the paths for the animation effect
        let mut prefix_paths        = vec![];
        let mut most_recent_paths   = vec![];
        let mut most_recent_time    = Duration::from_millis(0);

        for path in content.paths() {
            // Ignore any path that appears after the specified time
            if path.appearance_time > time { continue; }

            // Paths at time 0 go in the prefix if we're using the 'initial' effect
            if path.appearance_time == Duration::from_millis(0) && effect == FrameByFrameEffect::AddToInitial {
                prefix_paths.push(path.clone());
                continue;
            }

            // Ignore paths that occur before the most recent frame time
            if path.appearance_time < most_recent_time {
                continue;
            }

            // Clear the paths if we're closer to the requested time
            if path.appearance_time > most_recent_time {
                most_recent_time = path.appearance_time;
                most_recent_paths.clear();
            }

            // Add to the paths list
            most_recent_paths.push(path.clone());
        }

        // Create a new region content
        let content = AnimationRegionContent::from_paths(prefix_paths.into_iter().chain(most_recent_paths));

        Arc::new(content)
    }
}

impl AnimationEffect for FrameByFrameEffect {
    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        Self::animate_effect(*self, &region_contents, time)
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        let effect = *self;

        Box::new(move |time| {
            Self::animate_effect(effect, &region_contents, time)
        })
    }
}
