use ::serde::*;

use flo_scene::*;
use flo_draw::canvas::*;
use flo_binding::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Implementation of the notifiable interface for a scene
///
pub struct NotifySubprogram {
    /// The scene context that should be notified
    context: SceneContext,

    /// Sends the message for this notification
    send_message: Box<dyn Send + Sync + Fn(SceneContext) -> ()>,
}

impl NotifySubprogram {
    ///
    /// Creates a notification that will send a message to a target in the scene
    ///
    pub fn send<TMessage>(message: TMessage, context: &SceneContext, target: impl Into<StreamTarget>) -> Arc<Self>
    where
        TMessage : SceneMessage 
    {
        // Store the message for sending later
        let message         = Mutex::new(Some(message));

        // When the notification arrives, we spawn a command in the context to cause the message to be sent
        let target          = target.into();
        let send_message    = move |context: SceneContext| {
            if let Some(message) = message.lock().unwrap().take() {
                context.spawn_command(SendMessageCommand(Arc::new(Mutex::new(Some(message))), target.clone()), stream::empty()).ok();
            }
        };

        // Store the context and message sender in this object
        let notify = NotifySubprogram {
            context:        context.clone(),
            send_message:   Box::new(send_message),
        };

        Arc::new(notify)
    }
}

impl Notifiable for NotifySubprogram {
    fn mark_as_changed(&self) {
        (self.send_message)(self.context.clone())   
    }
}

///
/// Command that sends a message
///
struct SendMessageCommand<TMessage>(Arc<Mutex<Option<TMessage>>>, StreamTarget);

impl<TMessage> Command for SendMessageCommand<TMessage>
where
    TMessage: SceneMessage,
{
    type Input  = ();
    type Output = ();

    fn run<'a>(&'a self, _input: impl 'static + Send + Stream<Item=()>, context: SceneContext) -> impl 'a + Send + Future<Output=()> {
        let SendMessageCommand(message, target) = self;
        let message                             = message.lock().unwrap().take();

        async move {
            if let Some(message) = message {
                if let Some(mut target) = context.send(target.clone()).ok() {
                    target.send(message).await.ok();
                }
            }
        }
    }
}

impl<TMessage> Clone for SendMessageCommand<TMessage> {
    #[inline]
    fn clone(&self) -> Self {
        SendMessageCommand(Arc::clone(&self.0), self.1.clone())
    }
}
