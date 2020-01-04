use flo_commands::*;

use tokio::prelude::*;
use tokio::io::{stdout};
use futures::prelude::*;


#[tokio::main]
async fn main() {
    tokio::spawn(async {
        // Get the input commands
        let input       = stream::iter(vec![FloCommand::Version]);

        // Basic loop with a character output
        let mut stdout  = stdout();

        // Get the output stream
        let mut output  = to_char_output(flo_run_commands(input), 80);

        // Write the output to the stream
        while let Some(output_chr) = output.next().await {
            let mut bytes   = [0u8; 4];
            let byte_slice  = output_chr.encode_utf8(&mut bytes);
            stdout.write(byte_slice.as_bytes()).await.unwrap();
        }

        // Always finish with a newline
        stdout.write(&[10u8]).await.unwrap();
    }).await.unwrap();
}
