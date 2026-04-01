use crate::region::*;

use std::sync::*;
use std::time::{Duration};

///
/// Animation effect that applies a series of effects in sequence
///
pub struct SequenceEffect {
    /// The sequence that will be animated for this effect
    sequence: Arc<Mutex<Vec<Box<dyn AnimationEffect>>>>
}

impl SequenceEffect {
    ///
    /// Creates an empty sequence effect (which will leave the animation unaffected by default)
    ///
    pub fn empty() -> SequenceEffect {
        SequenceEffect {
            sequence: Arc::new(Mutex::new(vec![]))
        }
    }

    ///
    /// Adds a new effect to this sequence
    ///
    pub fn add_effect<TEffect: 'static+AnimationEffect>(&mut self, effect: TEffect) {
        self.sequence.lock().unwrap().push(Box::new(effect));
    }

    ///
    /// Adds an animation effect that's already boxed to this sequence
    ///
    pub fn add_boxed_effect(&mut self, effect: Box<dyn AnimationEffect>) {
        self.sequence.lock().unwrap().push(effect);
    }
}

impl AnimationEffect for SequenceEffect {
    ///
    /// Returns the duration of this effect (or None if this effect will animate forever)
    ///
    /// If the effect is passed a time that's after where the 'duration' has completed it should always generate the same result
    ///
    fn duration(&self) -> Option<f64> {
        // We take as long as the shortest effect in the sequence
        let mut shortest_time = None;

        for effect in self.sequence.lock().unwrap().iter() {
            if let Some(time) = effect.duration() {
                shortest_time = shortest_time
                    .map(|existing_time| if time < existing_time { time } else { existing_time })
                    .or(Some(time));
            }
        }

        shortest_time
    }

    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        // Apply each effect in turn to the sequence
        let sequence            = self.sequence.lock().unwrap();
        let mut region_contents = region_contents;

        for effect in sequence.iter() {
            region_contents = effect.animate(region_contents, time);
        }

        region_contents
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        // We can cache the first item in the sequence only, but need to call animate() on the future elements as they may get different results
        let sequence            = self.sequence.lock().unwrap();
        if sequence.len() > 0 {
            // Cache the first element
            // TODO: we could cache more elements if we know that they aren't affected by time
            let first_element   = sequence[0].animate_cached(region_contents);

            // Return a funciton that animates the whole sequence
            let sequence        = Arc::clone(&self.sequence);

            Box::new(move |time| {
                let sequence            = sequence.lock().unwrap();
                let mut region_contents = first_element(time);

                for effect in sequence.iter().skip(1) {
                    region_contents = effect.animate(region_contents, time);
                }

                region_contents
            })
        } else {
            // No animation to do!
            Box::new(move |_| Arc::clone(&region_contents))
        }
    }
}