use flo_commands::*;
use flo_animation::*;

use tokio::prelude::*;
use tokio::io::{stdin, stderr};
use tokio::fs;
use futures::prelude::*;
use clap::{App, Arg, SubCommand};

mod console;
use self::console::*;

use std::str::{FromStr};

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
        .arg(Arg::with_name("input-from-catalog")
            .long("input-from-catalog")
            .short("i")
            .takes_value(true)
            .help("Specifies a catalog file to use as the input animation (use the 'ls' command to see the catalog)"))
        .arg(Arg::with_name("catalog")
            .long("catalog")
            .short("C")
            .takes_value(true)
            .help("Specifies the directory where the catalog can be found"))
        .arg(Arg::with_name("output-to-catalog")
            .long("output-to-catalog")
            .short("W")
            .takes_value(true)
            .help("Creates a new animation in the catalog to use as the output target for this command"))
        .arg(Arg::with_name("input-from-file")
            .long("input-from-file")
            .short("I")
            .takes_value(true)
            .help("Specifies the path of a file to load as the input file"))
        .arg(Arg::with_name("frame")
            .long("frame")
            .short("F")
            .takes_value(true)
            .help("Specifies the layer and frame to apply the operation to (eg: -F 3:5 selects layer 3, frame 5)"))
        .subcommand(SubCommand::with_name("ls")
            .about("Lists animations in the main index"))
        .subcommand(SubCommand::with_name("ls-layers")
            .about("Lists the layers defined in the input animation"))
        .subcommand(SubCommand::with_name("ls-elements")
            .about("Lists all of the elements in the selected frame"))
        .subcommand(SubCommand::with_name("summarize-edits")
            .about("Reads all of the edits in the input animation and shows a summary of them"))
        .subcommand(SubCommand::with_name("rewrite-edits")
            .about("Reads all of the edits in the input animation and writes them to the output animation"))
        .subcommand(SubCommand::with_name("serialize-edits")
            .about("Reads all of the edits in the input animation and writes their serialized equivalent to the output"))
        .subcommand(SubCommand::with_name("deserialize-edits")
            .arg(Arg::with_name("INPUT")
                .help("The file to read from")
                .required(false)
                .index(1))
            .about("Reads a file (or standard input if no file is specified) containing serialized edits and writes them to the output animation"))
        .subcommand(SubCommand::with_name("dump-all-catalog-edits")
            .about("Writes out the entire catalog as a set of edit logs"))
        .subcommand(SubCommand::with_name("debug-raycasting")
            .about("Writes out a series of SVG files showing the raycasting used for a particular element")
            .arg(Arg::with_name("ELEMENT")
                .help("The element ID in the selected frame to raycast")
                .required(true)
                .index(1)))
        .get_matches();

    tokio::spawn(async move {
        // Get the input commands by parsing the parameters
        let mut input   = vec![];

        if params.is_present("version") {
            input.push(FloCommand::Version)
        }

        // Set the catalog folder if one is specified
        if let Some(catalog_folder) = params.value_of("catalog") {
            input.push(FloCommand::SetCatalogFolder(catalog_folder.to_string()));
        }

        // Read the input animation if one is specified
        if let Some(catalog_name) = params.value_of("input-from-catalog") {
            input.push(FloCommand::ReadFrom(StorageDescriptor::parse_catalog_string(catalog_name)));
        }

        if let Some(file_name) = params.value_of("input-from-file") {
            input.push(FloCommand::ReadFrom(StorageDescriptor::File(file_name.to_string())));
        }

        // Generate the output animation if there is one
        if let Some(name) = params.value_of("output-to-catalog") {
            input.push(FloCommand::WriteToCatalog(name.to_string()));
        }

        // Pick a frame if the user wanted one
        if let Some(frame) = params.value_of("frame") {
            // Expect two numbers seperated by a ':'
            if let Some(sep_pos) = frame.find(':') {
                // Split into layer and frame
                let (layer_num, frame_num) = (frame[0..sep_pos].to_string(), frame[sep_pos+1..frame.len()].to_string());

                if let (Ok(layer_num), Ok(frame_num)) = (u64::from_str(&layer_num), usize::from_str(&frame_num)) {
                    // Request this frame in the input
                    input.push(FloCommand::SelectFrame(layer_num, frame_num));
                } else {
                    // Bad frame format
                    stderr().write(format!("'{}:{}' is not a valid value for --frame. The parameter must be of the format <layer_id>:<frame_number> (eg: 3:5 for frame 5 of layer 3)\n\n", layer_num, frame_num).as_bytes()).await.unwrap();
                    return;
                }
            } else {
                // Bad frame format
                stderr().write(format!("'{}' is not a valid value for --frame. The parameter must be of the format <layer_id>:<frame_number> (eg: 3:5 for frame 5 of layer 3)\n\n", frame).as_bytes()).await.unwrap();
                return;
            }
        }

        // Ls command
        if let Some(_ls_params) = params.subcommand_matches("ls") {
            input.push(FloCommand::ListAnimations);
        }

        // Ls-layers command
        if let Some(_ls_params) = params.subcommand_matches("ls-layers") {
            input.push(FloCommand::ListLayers);
        }

        // Ls-elements command
        if let Some(_ls_params) = params.subcommand_matches("ls-elements") {
            input.push(FloCommand::ListElements);
        }

        // Summarize edits command
        if let Some(_) = params.subcommand_matches("summarize-edits") {
            input.push(FloCommand::ReadAllEdits);
            input.push(FloCommand::SummarizeEdits);
        }

        // Write edits command
        if let Some(_) = params.subcommand_matches("rewrite-edits") {
            input.push(FloCommand::ReadAllEdits);
            input.push(FloCommand::WriteAllEdits);

            input.push(FloCommand::ClearEdits);
            input.push(FloCommand::ReadFromWriteAnimation);
            input.push(FloCommand::ReadAllEdits);
            input.push(FloCommand::SummarizeEdits);
        }

        // Dump catalog command
        if let Some(_) = params.subcommand_matches("dump-all-catalog-edits") {
            input.push(FloCommand::DumpCatalogAsEdits);
        }

        // Serialize edits command
        if let Some(_) = params.subcommand_matches("serialize-edits") {
            input.push(FloCommand::ReadAllEdits);
            input.push(FloCommand::SerializeEdits);
        }

        // Deserialize edits command
        if let Some(deserialize) = params.subcommand_matches("deserialize-edits") {
            // Read the input file
            let mut input_data;
            if let Some(input_file) = deserialize.value_of("INPUT") {
                input_data = fs::read_to_string(input_file).await.unwrap();
            } else {
                input_data = String::new();
                stdin().read_to_string(&mut input_data).await.unwrap();
            }

            input.push(FloCommand::DeserializeEdits(input_data));
            input.push(FloCommand::WriteAllEdits);
        }

        // Debug raycasting command
        if let Some(debug_raycasting) = params.subcommand_matches("debug-raycasting") {
            // Parse the element ID
            let element_id = if let Some(element_id) = debug_raycasting.value_of("ELEMENT") {
                i64::from_str(element_id).ok()
            } else {
                None
            };
            let element_id = match element_id {
                Some(id)    => ElementId::Assigned(id),
                None        => { 
                    stderr().write(format!("'{}' is not a valid element ID\n\n", debug_raycasting.value_of("ELEMENT").unwrap_or("-")).as_bytes()).await.unwrap();
                    return;
                }
            };

            // Add a raycast command
            input.push(FloCommand::RayCastToSvg(element_id));
        }
        
        // Prepare as a stream as input to the command line
        let input       = stream::iter(input);

        // Basic loop with a character output
        let mut stderr  = stderr();

        // Write the output to the stream
        run_console(flo_run_commands(input)).await;

        // Always finish with a newline
        stderr.write(&[10u8]).await.unwrap();
    }).await.unwrap();
}
