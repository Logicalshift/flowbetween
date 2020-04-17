use super::state::*;
use super::error::*;
use super::output::*;
use super::command::*;
use super::subcommands::*;

use flo_stream::*;
use futures::prelude::*;
use futures::stream;
use futures::task::{Poll};

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
/// Runs a single command
///
fn run_command<'a>(command: FloCommand, output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+'a {
    async move {
        // Commands begin and end with a 'begin/finish' output
        output.publish(FloCommandOutput::BeginCommand(command.clone())).await;

        // Dispatch the command action
        match command {
            FloCommand::Version                     =>  { 
                let msg = format!("{} ({}) v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_VERSION"));

                output.publish(FloCommandOutput::Message(msg)).await;
            }

            FloCommand::ReadState                       => { output.publish(FloCommandOutput::State(state.clone())).await; }
            FloCommand::SetState(ref new_state)         => { *state = new_state.clone(); }

            FloCommand::SetCatalogFolder(ref folder)    => { set_catalog_folder(folder, output, state).await?; }

            FloCommand::ListAnimations                  => { list_files(output, state).await; }
            FloCommand::ReadFrom(ref read_location)     => { read_from(read_location.clone(), output, state).await?; }
            FloCommand::WriteToCatalog(ref name)        => { write_to_catalog(name.clone(), output, state).await?; }
            FloCommand::ReadFromWriteAnimation          => { *state = state.read_from_write_side(); }
            FloCommand::ReadAllEdits                    => { read_all_edits(output, state).await?; }
            FloCommand::SummarizeEdits                  => { summarize_edit_log(output, state).await?; }
            FloCommand::WriteAllEdits                   => { write_all_edits(output, state).await?; }
            FloCommand::SerializeEdits                  => { serialize_edits(output, state).await?; }
            FloCommand::ClearEdits                      => { *state = state.clear_edit_buffer(); }
            FloCommand::DumpCatalogAsEdits              => { dump_catalog_as_edits(output, state).await; }
            FloCommand::DeserializeEdits(ref edits)     => { deserialize_edits(stream::iter(edits.chars()), output, state).await?; }
            FloCommand::ListLayers                      => { list_layers(output, state).await; }
            FloCommand::SelectFrame(layer, when)        => { select_frame(output, state, layer, when).await; }
            FloCommand::ListElements                    => { list_elements(output, state).await; }
            FloCommand::RayCastToSvg(element_id)        => { raycast_to_svg(output, state, element_id).await?; }
        }

        // Finish the command
        output.publish(FloCommandOutput::FinishCommand(command.clone())).await;

        Ok(())
    }
}

///
/// Runs the specified series of commands and writes the output to the given publisher
///
fn run_commands<InputStream>(mut commands: InputStream, mut output: Publisher<FloCommandOutput>) -> impl Future<Output=()>+Send
where InputStream: 'static+Stream<Item=FloCommand>+Send+Unpin {
    // Create the initial state of the command
    let mut state = CommandState::new();

    async move {
        while let Some(command) = commands.next().await {
            // Run the next command
            match run_command(command, &mut output, &mut state).await {
                Ok(())      => { }
                Err(err)    => {
                    // Stop running commands if we get an error
                    output.publish(FloCommandOutput::Failure(err)).await;
                    break;
                }
            }
        }
    }
}
