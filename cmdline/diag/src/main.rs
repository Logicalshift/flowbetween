use flo_commands::*;

use tokio::prelude::*;
use tokio::io::{stdout};
use futures::prelude::*;
use clap::{App, Arg};

#[tokio::main]
async fn main() {
    // Fetch the parameters
    let params = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Copyright 2017-2020 Andrew Hunter <andrew@logicalshift.io>")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .after_help(concat!("Full source code is available at https://github.com/Logicalshift/flowbetween\n",
            "\n",
            "Licensed under the Apache License, Version 2.0 (the \"License\");\n",
            "you may not use this file except in compliance with the License.\n",
            "You may obtain a copy of the License at\n",
            "\n",
            "http://www.apache.org/licenses/LICENSE-2.0\n\n"))
        .arg(Arg::with_name("version")
            .long("version")
            .help("Displays version information"))
        .get_matches();

    tokio::spawn(async move {
        // Get the input commands by parsing the parameters
        let mut input   = vec![];

        if params.is_present("version") {
            input.push(FloCommand::Version)
        }

        // Prepare as a stream as input to the command line
        let input       = stream::iter(input);

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
