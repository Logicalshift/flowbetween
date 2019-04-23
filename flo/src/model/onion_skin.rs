use flo_canvas::*;
use flo_binding::*;

use std::time::Duration;

///
/// 
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
pub struct OnionSkinModel {
    /// Whether or not the onion skins should be displayed
    pub show_onion_skins: Binding<bool>,

    /// The drawing actions for the onion skins to display
    pub onion_skins: BindRef<Vec<(OnionSkinTime, Vec<Draw>)>>,

    /// The number of frames to show before the current frame
    pub frames_before: Binding<usize>,

    /// The number of frames to show after the current frame
    pub frames_after: Binding<usize>,

    /// The times of the onion skins to display
    pub onion_skin_times: BindRef<OnionSkinTime>
}

impl OnionSkinModel {
    ///
    /// Creates a new onion skin model
    ///
    pub fn new() -> OnionSkinModel {
        // Create the basic bindings
        let show_onion_skins    = Binding::new(false);
        let frames_before       = Binding::new(3);
        let frames_after        = Binding::new(3);

        // Create the derived bindings
        let onion_skin_times    = Self::onion_skin_times();
        let onion_skins         = Self::onion_skins();

        OnionSkinModel {
            show_onion_skins,
            frames_before,
            frames_after,
            onion_skin_times,
            onion_skins
        }
    }

    ///
    /// Returns the current set of times to display onion skins for
    ///
    fn onion_skin_times() -> BindRef<OnionSkinTime> {
        unimplemented!()
    }

    ///
    /// Returns a binding for the set of drawing actions to draw the current set of onion skins
    ///
    fn onion_skins() -> BindRef<Vec<(OnionSkinTime, Vec<Draw>)>> {
        unimplemented!()
    }
}