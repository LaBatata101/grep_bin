use clap::{values_t, App, AppSettings, Arg, ArgMatches};
use std::fs::File;
use std::io::BufReader;
use std::process;
use std::{env, path::PathBuf};

use ansi_term::Colour;
pub mod utils;

pub use utils::{file, search};

use crate::utils::{print_output, PatternType};


pub fn setup_args<'a>() -> ArgMatches<'a> {
    App::new("grep_bin")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .long_about(
            "Searches for a sequence of bytes  or a ASCII string in a binary file.
If a directory is provided grep_bin will search every file in the directory recursively.",
        )
        .arg(
            Arg::with_name("FILE")
                .index(1)
                .required(true)
                .multiple(true)
                .empty_values(false)
                .help("The file path"),
        )
        .arg(
            Arg::with_name("PATTERN")
                .index(2)
                .required(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("filetype")
                .short("f")
                .multiple(true)
                .takes_value(true)
                .empty_values(false)
                .long_help(
                    "Filter the search by the file extensions.
Examples of input: jpg, mp3, exe",
                ),
        )
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .get_matches()
}

pub fn parse_args(args: ArgMatches) {
    let filetypes: Vec<&str> = args.values_of("filetype").unwrap_or_default().collect();

    let filepaths = values_t!(args, "FILE", PathBuf).unwrap();
    let files: Vec<PathBuf> = if filetypes.is_empty() {
        file::get_all_files_from_paths(filepaths)
    } else {
        file::filter_filetypes(file::get_all_files_from_paths(filepaths), &filetypes)
    };

    let bytes: Vec<u8> = match PatternType::from(args.value_of("PATTERN").unwrap()) {
        PatternType::Str(pattern) => pattern.to_owned().into_bytes(),

        PatternType::HexStr(pattern) => hex::decode(pattern).unwrap_or_else(|error| {
            eprintln!("Error: {} in byte sequence!", error);
            process::exit(1);
        }),
    };

    for filename in files {
        let file = File::open(&filename).unwrap_or_else(|error| {
            eprintln!("{}: {}", filename.to_str().unwrap(), error);
            process::exit(1);
        });
        let reader = BufReader::new(file);

        println!("{}", Colour::Purple.paint(filename.to_str().unwrap()));
        let matches = search::search_in_file(&bytes, reader);

        for _match in matches {
            print_output(_match);
        }
    }
}
