mod timeline;
mod brush;
mod tools;
mod menu;
mod layer;
mod keyframe;
mod animation;

pub use self::timeline::*;
pub use self::brush::*;
pub use self::tools::*;
pub use self::menu::*;
pub use self::layer::*;
pub use self::keyframe::*;
pub use self::animation::*;

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
    timeline: TimelineViewModel<Anim>,

    /// The brush view model
    brush: BrushViewModel,

    /// The view model for the menu bar
    menu: MenuViewModel
}

impl<Anim: Animation+'static> AnimationViewModel<Anim> {
    ///
    /// Creates a new view model
    /// 
    pub fn new(animation: Anim) -> AnimationViewModel<Anim> {
        let animation   = Arc::new(animation);
        let tools       = ToolViewModel::new();
        let timeline    = TimelineViewModel::new(Arc::clone(&animation));
        let brush       = BrushViewModel::new();
        let menu        = MenuViewModel::new(&tools.effective_tool);

        AnimationViewModel {
            animation:      animation,
            tools:          tools,
            timeline:       timeline,
            brush:          brush,
            menu:           menu
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
    pub fn timeline(&self) -> &TimelineViewModel<Anim> {
        &self.timeline
    }

    ///
    /// Retrieves the viewmodel of the brush settings for this animation
    /// 
    pub fn brush(&self) -> &BrushViewModel {
        &self.brush
    }

    ///
    /// Retrieves the viewmodel for the menu for this animation
    /// 
    pub fn menu(&self) -> &MenuViewModel {
        &self.menu
    }
}

// Clone because for some reason #[derive(Clone)] does something weird
impl<Anim: Animation> Clone for AnimationViewModel<Anim> {
    fn clone(&self) -> AnimationViewModel<Anim> {
        AnimationViewModel {
            animation:      self.animation.clone(),
            tools:          self.tools.clone(),
            timeline:       self.timeline.clone(),
            brush:          self.brush.clone(),
            menu:           self.menu.clone()
        }
    }
}
