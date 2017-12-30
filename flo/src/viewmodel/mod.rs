mod timeline;
mod tools;

pub use self::timeline::*;
pub use self::tools::*;

use animation::*;

use std::sync::*;

///
/// The viewmodel for the animation editor
/// 
pub struct AnimationViewModel<Anim: Animation> {
    /// The animation that is being edited
    animation: Arc<Anim>,

    /// The status of the currently selected tool
    tools: ToolViewModel<Anim>,

    /// The timeline view model
    timeline: TimelineViewModel
}

impl<Anim: Animation+'static> AnimationViewModel<Anim> {
    ///
    /// Creates a new view model
    /// 
    pub fn new(animation: Anim) -> AnimationViewModel<Anim> {
        AnimationViewModel {
            animation:      Arc::new(animation),
            tools:          ToolViewModel::new(),
            timeline:       TimelineViewModel::new()
        }
    }

    ///
    /// Retrieves the animation being edited by this viewmodel
    /// 
    pub fn animation(&self) -> &Anim {
        &*self.animation
    }

    ///
    /// Retrieves a reference to the animation being edited by this viewmodel
    /// 
    pub fn animation_ref(&self) -> Arc<Anim> {
        Arc::clone(&self.animation)
    }

    ///
    /// Retrieves the viewmodel for the drawing tools for this animation
    /// 
    pub fn tools(&self) -> &ToolViewModel<Anim> {
        &self.tools
    }

    ///
    /// Retrieves the viewmodel of the timeline for this animation
    /// 
    pub fn timeline(&self) -> &TimelineViewModel {
        &self.timeline
    }
}

// Clone because for some reason #[derive(Clone)] does something weird
impl<Anim: Animation> Clone for AnimationViewModel<Anim> {
    fn clone(&self) -> AnimationViewModel<Anim> {
        AnimationViewModel {
            animation:      self.animation.clone(),
            tools:          self.tools.clone(),
            timeline:       self.timeline.clone()
        }
    }
}
