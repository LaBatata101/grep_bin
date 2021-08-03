use clap::{App, AppSettings, Arg, ArgMatches};
use grep_bin::{search, util};
use std::path::PathBuf;
use std::process;

fn main() {
    let args = setup_args();
    parse_args(args);
}

fn setup_args<'a>() -> ArgMatches<'a> {
    App::new("grep_bin")
        .version(clap::crate_version!())
        .long_about(
            "Searches for a sequence of bytes  or a ASCII string in a binary file.
If a directory is provided grep_bin will search every file in the directory recursively.",
        )
        .arg(
            Arg::with_name("filepath")
                .short("f")
                .required(true)
                .multiple(true)
                .takes_value(true)
                .empty_values(false)
                .help("The file path"),
        )
        .arg(
            Arg::with_name("filetype")
                .short("t")
                .multiple(true)
                .takes_value(true)
                .empty_values(false)
                .long_help(
                    "Filter the search by the file extensions.
Examples of input: jpg, mp3, exe",
                ),
        )
        .arg(
            Arg::with_name("bytes")
                .short("b")
                .requires("filepath")
                .required_unless("string")
                .max_values(1)
                .required(true)
                .takes_value(true)
                .empty_values(false)
                .long_help(
                    "The sequence of bytes to be searched in the file.
Example of valid inputs: f9b4ca, F9B4CA and f9B4Ca are all valid.",
                ),
        )
        .arg(
            Arg::with_name("string")
                .short("s")
                .requires("filepath")
                .required_unless("bytes")
                .takes_value(true)
                .max_values(1)
                .empty_values(false)
                .help("Search for ASCII string inside the file"),
        )
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .get_matches()
}

fn parse_args(args: ArgMatches) {
    let filetypes: Vec<&str> = args.values_of("filetype").unwrap_or_default().collect();

    let filepaths = args.values_of("filepath").unwrap().collect();
    let files: Vec<PathBuf> = if filetypes.is_empty() {
        util::get_all_files_from_paths(filepaths)
    } else {
        util::filter_filetypes(util::get_all_files_from_paths(filepaths), &filetypes)
    };

    if args.is_present("string") {
        let string_to_search = args.value_of("string").unwrap();

        let bytes_to_search =
            match util::string_to_bytes(&string_to_search.chars().collect::<Vec<char>>()) {
                Ok(bytes) => bytes,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            };
        search::search_in_files(&bytes_to_search, &files);
    }

    if args.is_present("bytes") {
        let pattern = args.value_of("bytes").unwrap();

        let bytes_to_search = match hex::decode(pattern) {
            Ok(hex) => hex,
            Err(err) => {
                eprintln!("Error: {} in byte sequence!", err);
                process::exit(1);
            }
        };
        search::search_in_files(&bytes_to_search, &files);
    }
}
