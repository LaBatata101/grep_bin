use clap::{arg, command, ArgMatches};
use std::process;
use std::{env, path::PathBuf};

use ansi_term::Colour;
mod utils;

use utils::{file, search};

use crate::utils::{print_hexdump_output, PatternType};

pub fn setup_args() -> ArgMatches {
    let integer_validator = |value: &str| match value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("the value needs to be a valid integer")),
    };

    command!()
        .arg(
Ascii strings should be passed inside quotes like so '\"This is a string\"'
Escaping quotes '\"This is a \\\"quoted string\\\"\"'
All of these byte sequence are valid: f9b4ca, F9B4CA and f9B4Ca")
        )
        .arg(
            arg!(-f <filetype> ... "Filter the search by the file extensions.
Examples of input: jpg, mp3, exe")
            .required(false),
        )
        .arg(
            arg!(-c <context_bytes_size> "Defines the number of bytes that will be printed in each line.")
                .required(false)
                .default_value("16")
                .validator(integer_validator),
        )
        .arg(
            arg!(-p --"print-only" "Prints only the filename that contais the match.")
                .id("print_only")
                .requires("PATTERN"),
        )
        .arg(
            arg!(-o --"print-offset" "Prints only the offsets of the match.")
                .id("print_offset")
                .requires("PATTERN"),
        )
        .arg(
            arg!(-s --"skip-bytes" <n> "Skip n bytes before searching.")
                .id("skip_bytes")
                .required(false)
                .default_value("0")
                .validator(integer_validator),
        )
        .arg_required_else_help(true)
        .get_matches()
}

pub fn run(args: ArgMatches) {
    let filetypes: Vec<&str> = args.values_of("filetype").unwrap_or_default().collect();

    let filepaths: Vec<PathBuf> = args.values_of_t("FILE").unwrap();
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
