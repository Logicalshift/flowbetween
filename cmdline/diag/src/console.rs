use flo_commands::*;

use tokio::prelude::*;
use tokio::prelude::{AsyncWrite};
use tokio::fs;
use tokio::io::{stdout, stderr};
use futures::prelude::*;

use std::path::*;

///
/// Sends the command output to the console
///
pub fn run_console<InputStream>(command_output: InputStream) -> impl Future<Output=()>+Send
where InputStream: Stream<Item=FloCommandOutput>+Send+Unpin {
    async move {
        let mut command_output = command_output;

        // The default command output stream is stdout. The messages are sent to stderr
        let mut output_stream: Box<dyn AsyncWrite+Send+Unpin>   = Box::new(stdout());
        let mut message_stream                                  = stderr();

        while let Some(input) = command_output.next().await {
            use self::FloCommandOutput::*;

            match input {
                BeginCommand(_cmd)              => { }
                Message(msg)                    => { message_stream.write(msg.as_bytes()).await.unwrap(); message_stream.write("\n".as_bytes()).await.unwrap(); }
                BeginOutput(filename)           => { output_stream = Box::new(fs::File::create(PathBuf::from(filename)).await.unwrap()); }
                Error(err)                      => { message_stream.write(err.as_bytes()).await.unwrap(); message_stream.write("\n".as_bytes()).await.unwrap(); }
                State(_state)                   => { }
                FinishCommand(_cmd)             => { }
                StartTask(_task)                => { }
                TaskProgress(_complete, _todo)  => { }
                FinishTask                      => { }
                Failure(error)                  => { 
                    let msg = format!("ERROR: {}", error);
                    message_stream.write(msg.as_bytes()).await.unwrap();
                }

                Output(output)                  => {
                    let bytes   = output.as_bytes();
                    let mut pos = 0;

                    while pos < bytes.len() {
                        let remaining_bytes = &bytes[pos..bytes.len()];

                        let num_written     = output_stream.write(remaining_bytes).await.unwrap();
                        pos                 += num_written;
                    }
                }
            }
        }
    }
}
