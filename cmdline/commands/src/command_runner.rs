use super::output::*;
use super::command::*;
use super::subcommands::*;

use flo_stream::*;
use futures::prelude::*;
use futures::stream;
use futures::task::{Poll};

/// The name of the app (after our domain: flowbetween.app)
pub const APP_NAME: &str = "app.flowbetween";

/// Where we store the default user data
pub const DEFAULT_USER_FOLDER: &str = "default";

///
/// Runs a series of commands provided by a stream and returns a stream of the resulting output
///
pub fn flo_run_commands<InputStream>(commands: InputStream) -> impl Stream<Item=FloCommandOutput>+Send+Unpin
where InputStream: 'static+Stream<Item=FloCommand>+Unpin+Send {
    // Create the output
    let mut output_publisher    = Publisher::new(1);
    let mut output              = output_publisher.subscribe();
    let mut runner              = Some(run_commands(commands, output_publisher).boxed());

    // Reading from the output stream causes commands to be run
    stream::poll_fn(move |context| {
        // Try to run a command
        if let Some(ref mut active_runner) = runner {
            if active_runner.poll_unpin(context) == Poll::Ready(()) {
                // Command has completed: free up the runner
                runner = None;
            }
        }

        // Try to read some output. We stop running when the output stream is no longer being read from
        output.poll_next_unpin(context)
    })
}

///
/// Runs the specified series of commands and writes the output to the given publisher
///
fn run_commands<InputStream>(mut commands: InputStream, mut output: Publisher<FloCommandOutput>) -> impl Future<Output=()>+Send
where InputStream: 'static+Stream<Item=FloCommand>+Send+Unpin {
    async move {
        while let Some(command) = commands.next().await {
            // Commands begin and end with a 'begin/finish' output
            output.publish(FloCommandOutput::BeginCommand(command.clone())).await;

            match command {
                FloCommand::Version                     =>  { 
                    let msg = format!("{} ({}) v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_VERSION"));

                    output.publish(FloCommandOutput::Message(msg)).await;
                }

                FloCommand::ListAnimations              => { list_files(&mut output, APP_NAME.to_string(), DEFAULT_USER_FOLDER.to_string()).await }
                FloCommand::ReadFrom(ref read_location) => { unimplemented!() }
                FloCommand::WriteTo(ref write_location) => { unimplemented!() }
                FloCommand::ReadAllEdits                => { unimplemented!() }
                FloCommand::SummarizeEdits              => { unimplemented!() }
            }

            // Finish the command
            output.publish(FloCommandOutput::FinishCommand(command.clone())).await;
        }
    }
}
