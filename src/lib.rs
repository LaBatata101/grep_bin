use clap::{values_t, App, AppSettings, Arg, ArgMatches};
use std::process;
use std::{env, path::PathBuf};

use ansi_term::Colour;
mod utils;

use utils::{file, search};

use crate::utils::{print_hexdump_output, PatternType};

pub fn setup_args<'a>() -> ArgMatches<'a> {
    let integer_validator = |value: String| match value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("the value needs to be a valid integer")),
    };

    App::new("grep_bin")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .long_about(clap::crate_description!())
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
                .empty_values(false)
                .long_help(
                    "Can be a ascii string or a byte sequence.
Ascii strings should be passed inside quotes like so '\"This is a string\"'
Escaping quotes '\"This is a \\\"quote\\\"\"'
All of these byte sequence are valid: f9b4ca, F9B4CA and f9B4Ca",
                ),
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
        .arg(
            Arg::with_name("context_bytes_size")
                .short("c")
                .default_value("16")
                .validator(integer_validator)
                .long_help("Defines the number of bytes that will be printed in each line."),
        )
        .arg(
            Arg::with_name("print_only")
                .short("p")
                .long("print-only")
                .help("Prints only the file name that contais the match."),
        )
        .arg(
            Arg::with_name("skip_bytes")
                .short("s")
                .long("skip-bytes")
                .default_value("0")
                .takes_value(true)
                .validator(integer_validator)
                .help("Skip n bytes before searching."),
        )
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .get_matches()
}

pub fn run(args: ArgMatches) {
    let filetypes: Vec<&str> = args.values_of("filetype").unwrap_or_default().collect();

    let filepaths = values_t!(args, "FILE", PathBuf).unwrap();
    let files: Vec<PathBuf> = if filetypes.is_empty() {
        file::get_all_files_from_paths(filepaths)
    } else {
        file::filter_filetypes(file::get_all_files_from_paths(filepaths), &filetypes)
    };

    let pattern: Vec<u8> = match PatternType::from(args.value_of("PATTERN").unwrap()) {
        PatternType::Str(pattern) => pattern.into_bytes(),

        PatternType::HexStr(pattern) => hex::decode(pattern).unwrap_or_else(|error| {
            eprintln!("Error: {} in byte sequence!", error);
            process::exit(1);
        }),
    };

    let context_bytes_size: usize = args
        .value_of("context_bytes_size")
        .unwrap()
        .parse()
        .unwrap();
    let skip_bytes: u64 = args.value_of("skip_bytes").unwrap().parse().unwrap();

    for filename in files {
        let mut searcher = search::Searcher::new(&pattern, context_bytes_size, skip_bytes);
        let filename = filename.to_str().unwrap();

        searcher.search_in_file(filename).unwrap_or_else(|error| {
            eprintln!("{}: {}", filename, error);
            process::exit(1);
        });

        let result = searcher.result();
        if !result.is_empty() {
            println!("{}", Colour::Purple.paint(filename));
        }

        if !args.is_present("print_only") {
            print_hexdump_output(result, searcher.context_bytes_size());
        }
    }
}
