use clap::{App, AppSettings, Arg};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let app = App::new("grep_bin")
        .version("0.1.0")
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
            Arg::with_name("recursive")
                .short("r")
                .help("Search recursivly in the file path"),
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .requires("filepath")
                .max_values(1)
                .required(true)
                .takes_value(true)
                .empty_values(false)
                .long_help(
                    "The sequence of bytes to be searched in file.
Example of valid inputs: f9b4ca, F9B4CA and f9B4Ca are all valid.",
                ),
        )
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp]);

    let matches = app.get_matches();

    let pattern = matches.value_of("search").unwrap();

    let mut files = Vec::new();
    for file in matches.values_of("filepath").unwrap() {
        let filepath = Path::new(file);

        if filepath.is_dir() {
            files.extend(get_all_files_from_path(filepath));
        }
        files.push(file.to_string());
    }

    let bytes_to_search = hex::decode(pattern).unwrap_or_else(|_| {
        eprintln!("Wrong format!");
        process::exit(1);
    });
            Ok(file) => BufReader::new(file),
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        };

        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .expect("Failed reading file to buffer!");

        let result = search_subslice(&buffer, &[0xFF, 0xFE]);
        print_hexdump(result, &buffer);
    }
}

fn print_hexdump(indexes: Vec<usize>, src: &[u8]) {
    let padding = 16; // 16 bytes per row

    for index in indexes {
        let offset = index - (index % padding);

        print!("{:08X}  ", offset);

        for (i, pos) in (offset..(offset + padding)).enumerate() {
            print!("{:02X} ", src[pos as usize]);

            if i == 7 {
                print!(" ");
            }

            std::io::stdout().flush().unwrap();

            if i == 15 {

                print!("  |");
                print_ascii_representation(&src[offset..(offset + padding)]);
                print!("|");

                std::io::stdout().flush().unwrap();

                println!();
            }
        }
    }
}

fn print_ascii_representation(chars: &[u8]) {
    for &c in chars {
        let ch = c as char;

        if ch.is_ascii() && !ch.is_ascii_control() {
            print!("{}", ch);
        } else {
            print!(".");
        }

        std::io::stdout().flush().unwrap();
    }
}

fn get_all_files_from_path(path: &Path) -> Vec<String> {
    let mut filepaths: Vec<String> = Vec::new();

    visit_dirs(path, &mut |file| {
        filepaths.push(file.to_str().unwrap_or_default().to_string())
    })
    .unwrap();

    filepaths
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(PathBuf)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(entry.path());
            }
        }
    }
    Ok(())
}

fn search_subslice(input: &[u8], pattern: &[u8]) -> Vec<usize> {
    let mut match_indexes: Vec<usize> = Vec::new();

    let mut curr_pos_pattern: usize = 0;
    let table_of_ocurrencies = pre_process(pattern);

    for (i, &ch) in input.iter().enumerate() {
        while curr_pos_pattern > 0 && pattern[curr_pos_pattern] != ch {
            curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern - 1];
        }

        if pattern[curr_pos_pattern] == ch {
            if curr_pos_pattern == pattern.len() - 1 {
                match_indexes.push(i - curr_pos_pattern);
                curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern];
            } else {
                curr_pos_pattern += 1;
            }
        }
    }

    match_indexes
}

fn pre_process(pattern: &[u8]) -> Vec<usize> {
    let mut table_of_ocurrencies: Vec<usize> = vec![0; pattern.len()];
    let mut pos = 0;

    for i in 1..pattern.len() {
        while pos > 0 && pattern[i] != pattern[pos] {
            pos = table_of_ocurrencies[pos - 1];
        }

        if pattern[pos] == pattern[i] {
            pos += 1;
            table_of_ocurrencies[i] = pos;
        }
    }

    table_of_ocurrencies
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tests_search() {
        let text = vec![0x00, 0x01, 0x00, 0xFF, 0xFE, 0x00, 0xA4, 0x00];
        assert_eq!(vec![3, 4], search_subslice(&text, &[0xFF, 0xFE, 0x00]));
    }

    #[test]
    fn test_string_search() {
        assert_eq!(
            vec![0, 9, 12],
            search_subslice(
                &[
                    b'A', b'A', b'B', b'A', b'A', b'C', b'A', b'A', b'D', b'A', b'A', b'B', b'A',
                    b'A', b'B', b'A'
                ],
                &[b'A', b'A', b'B', b'A']
            )
        )
    }
}
