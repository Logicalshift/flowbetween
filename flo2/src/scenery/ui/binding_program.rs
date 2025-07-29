use super::binding_tracker::*;

use flo_binding::*;
use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use ::serde::*;

use std::marker::{PhantomData};

///
/// Describes when the binding action will be carried out
///
#[derive(Clone, Copy, Debug)]
pub enum BindingTrigger {
    /// Perform the binding action as soon as the binding changes
    Immediate,

    /// Perform the binding action only once the scene is idle
    WaitForIdle,
}

///
/// Describes the action to take in a binding program
///
pub struct BindingAction<TValue, TFn, TFuture> {
    /// The action to take when the binding is changed
    action: TFn,

    /// We'll stop tracking the binding whenever this program finishes
    parent_program: Option<SubProgramId>,

    /// When the binding action should be performed
    trigger: BindingTrigger,

    /// Value and future are used by the function
    phantom: PhantomData<(TValue, TFuture)>,
}

///
/// Binding programs take an action every time the supplied binding changes, optionally when an idle request is made
///
pub async fn binding_program<TValue, TFn, TFuture>(
    input_stream:   InputStream<BindingProgram>, 
    context:        SceneContext, 
    binding:        impl Into<BindRef<TValue>>,
    action:         BindingAction<TValue, TFn, TFuture>)
where
    TFn:        Send + FnMut(TValue, &SceneContext) -> TFuture,
    TFuture:    Send + Future<Output=()>,
{
    let mut action  = action;
    let binding     = binding.into();

    // Need to know the program ID we've been assigned
    let our_program_id = context.current_program_id().unwrap();

    // Connect to the idle request program
    let mut idle_requests = context.send::<IdleRequest>(()).unwrap();

    // Subscribe to scene updates if there's a parent program
    if action.parent_program.is_some() {
        context.send_message(SceneControl::Subscribe(our_program_id.into())).await.ok();
    }

    // Run the binding action with the initial program
    let mut tracker     = Some(binding.when_changed(NotifySubprogram::send(BindingProgram::BindingChanged, &context, our_program_id)));
    let initial_value   = binding.get();
    (action.action)(initial_value, &context).await;

    let mut input_stream = input_stream;
    while let Some(msg) = input_stream.next().await {
        match msg {
            BindingProgram::BindingChanged => {
                // Finished with the tracker at this point
                if let Some(mut tracker) = tracker.take() {
                    tracker.done();
                }

                match action.trigger {
                    BindingTrigger::Immediate => {
                        // Read the binding immediately
                        tracker         = Some(binding.when_changed(NotifySubprogram::send(BindingProgram::BindingChanged, &context, our_program_id)));
                        let new_value   = binding.get();
                        (action.action)(new_value, &context).await;
                    }

                    BindingTrigger::WaitForIdle => {
                        // Request an idle notification to perform the action
                        idle_requests.send(IdleRequest::WhenIdle(our_program_id)).await.ok();
                    }
                }
            }

            BindingProgram::Idle => {
                // Callback from BindingChanged if we're configured to wait for idle messages: perform the action and wait for a new value
                tracker         = Some(binding.when_changed(NotifySubprogram::send(BindingProgram::BindingChanged, &context, our_program_id)));
                let new_value   = binding.get();
                (action.action)(new_value, &context).await;
            }

            BindingProgram::Update(SceneUpdate::Stopped(prog_id)) => {
                // Stop running this program if the parent program stops
                if Some(prog_id) == action.parent_program {
                    break;
                }
            }

            BindingProgram::Update(_) => { /* Other updates are ignored */ }
        }
    }
}

impl<TValue, TFn, TFuture> BindingAction<TValue, TFn, TFuture>
where
    TFn:        Send + FnMut(TValue, &SceneContext) -> TFuture,
    TFuture:    Send + Future<Output=()>,
{
    ///
    /// Creates a new binding action, with the specified action to perform when the bound value changes
    ///
    pub fn new(action: TFn) -> Self {
        BindingAction {
            action:         action,
            parent_program: None,
            trigger:        BindingTrigger::WaitForIdle,
            phantom:        PhantomData,
        }
    }

    ///
    /// Returns an updated action with the specified triggering style. This determines when the action will be performed after a notification that 
    /// something has changed.
    ///
    /// The default triggering style is 'WaitForIdle'.
    ///
    pub fn with_trigger(mut self, new_trigger: BindingTrigger) -> Self {
        self.trigger = new_trigger;
        self
    }

    ///
    /// Returns an updated binding action that will cause the binding program to stop when the parent program stops
    ///
    pub fn with_parent_program(mut self, parent_program: SubProgramId) -> Self {
        self.parent_program = Some(parent_program);
        self
    }
}

///
/// Messages that can be sent to the a binding program
///
/// There's no need to send these manually, they're all to do with managing the events generated by a binding.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BindingProgram {
    /// The scene is idle so any changes can be rendered
    Idle,

    /// The binding has changed
    BindingChanged,

    /// Something about the scene has changed
    Update(SceneUpdate),
}

impl SceneMessage for BindingProgram {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        // The binding programs can receive idle notifications and scene updates
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|scene_updates| scene_updates.map(|update| BindingProgram::Update(update)))), (), StreamId::with_message_type::<SceneUpdate>()).ok();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|idle_updates| idle_updates.map(|_: IdleNotification| BindingProgram::Idle))), (), StreamId::with_message_type::<IdleNotification>()).ok();
    }
}
