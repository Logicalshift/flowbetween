use super::timeline::*;

use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use std::marker::PhantomData;
use std::time::Duration;

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

///
/// The model used to describe which onion skins are being displayed
///
pub struct OnionSkinModel<Anim: Animation> {
    /// Whether or not the onion skins should be displayed
    pub show_onion_skins: Binding<bool>,

    /// The drawing actions for the onion skins to display
    pub onion_skins: BindRef<Vec<(OnionSkinTime, Vec<Draw>)>>,

    /// The number of frames to show before the current frame
    pub frames_before: Binding<usize>,

    /// The number of frames to show after the current frame
    pub frames_after: Binding<usize>,

    /// The times of the onion skins to display
    pub onion_skin_times: BindRef<OnionSkinTime>,

    anim: PhantomData<Anim>
}

impl<Anim: 'static+Animation> OnionSkinModel<Anim> {
    ///
    /// Creates a new onion skin model
    ///
    pub fn new(timeline: &TimelineModel<Anim>) -> OnionSkinModel<Anim> {
        // Create the basic bindings
        let show_onion_skins    = Binding::new(false);
        let frames_before       = Binding::new(3);
        let frames_after        = Binding::new(3);

        // Create the derived bindings
        let onion_skin_times    = Self::onion_skin_times(timeline, BindRef::from(&show_onion_skins), BindRef::from(&frames_before), BindRef::from(&frames_after));
        let onion_skins         = Self::onion_skins();

        OnionSkinModel {
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
    fn onion_skin_times(timeline: &TimelineModel<Anim>, show_onion_skins: BindRef<bool>, frames_before: BindRef<usize>, frames_after: BindRef<usize>) -> BindRef<OnionSkinTime> {
        unimplemented!()
    }

    ///
    /// Returns a binding for the set of drawing actions to draw the current set of onion skins
    ///
    fn onion_skins() -> BindRef<Vec<(OnionSkinTime, Vec<Draw>)>> {
        unimplemented!()
    }
}

impl<Anim: Animation> Clone for OnionSkinModel<Anim> {
    fn clone(&self) -> OnionSkinModel<Anim> {
        OnionSkinModel {
            show_onion_skins:   self.show_onion_skins.clone(),
            frames_before:      self.frames_before.clone(),
            frames_after:       self.frames_after.clone(),
            onion_skin_times:   self.onion_skin_times.clone(),
            onion_skins:        self.onion_skins.clone(),
            anim:               PhantomData
        }
    }
}
