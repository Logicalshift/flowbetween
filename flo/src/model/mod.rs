mod timeline;
mod tools;
mod layer;
mod keyframe;
mod frame;
mod animation;

pub use self::timeline::*;
pub use self::tools::*;
pub use self::layer::*;
pub use self::keyframe::*;
pub use self::frame::*;
pub use self::animation::*;

use binding::*;
use animation::*;

use std::sync::*;

///
/// The viewmodel for the animation editor
/// 
pub struct FloModel<Anim: Animation> {
    /// The animation that is being edited
    animation: Arc<Anim>,

    /// The status of the currently selected tool
    tools: ToolModel<Anim>,

    /// The timeline view model
    timeline: TimelineModel<Anim>,

    /// The frame view model
    frame: FrameModel,

    /// The size of the animation
    pub size: BindRef<(f64, f64)>,

    /// The underlying size binding
    size_binding: Binding<(f64, f64)>
}

impl<Anim: Animation+'static> FloModel<Anim> {
    ///
    /// Creates a new view model
    /// 
    pub fn new(animation: Anim) -> FloModel<Anim> {
        let animation       = Arc::new(animation);
        let tools           = ToolModel::new();
        let timeline        = TimelineModel::new(Arc::clone(&animation));
        let frame           = FrameModel::new(Arc::clone(&animation), BindRef::new(&timeline.current_time), BindRef::from(bind(0)));

        let size_binding    = bind(animation.size());

        FloModel {
            animation:      animation,
            tools:          tools,
            timeline:       timeline,
            frame:          frame,

            size:           BindRef::from(size_binding.clone()),
            size_binding:   size_binding
        }
    }

    ///
    /// Retrieves the viewmodel for the drawing tools for this animation
    /// 
    pub fn tools(&self) -> &ToolModel<Anim> {
        &self.tools
    }

    ///
    /// Retrieves the viewmodel of the timeline for this animation
    /// 
    pub fn timeline(&self) -> &TimelineModel<Anim> {
        &self.timeline
    }
}

// Clone because for some reason #[derive(Clone)] does something weird
impl<Anim: Animation> Clone for FloModel<Anim> {
    fn clone(&self) -> FloModel<Anim> {
        FloModel {
            animation:      self.animation.clone(),
            tools:          self.tools.clone(),
            timeline:       self.timeline.clone(),
            frame:          self.frame.clone(),

            size:           self.size.clone(),
            size_binding:   self.size_binding.clone()
        }
    }
}
