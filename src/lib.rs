use ansi_term::Colour;
use std::path::PathBuf;
use std::process;

pub mod util {
    use super::{Colour, PathBuf};
    use std::io::Write;
    use std::ops::Range;

    pub fn string_to_bytes(str: &[char]) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        for c in str {
            if c.is_ascii() {
                bytes.push(*c as u8);
            } else {
                return Err(format!("Invalid ASCII character \"{}\"!", c));
            }
        }

        Ok(bytes)
    }

    pub fn print_hexdump(indexes: Vec<usize>, src: &[u8], pattern_size: usize) {
        let padding = 16; // 16 bytes per row
        let src_len = src.len();

        for index in indexes {
            let offset = index - (index % padding);
            let indexes_to_paint = index..index + pattern_size;

            print!("{}:  ", Colour::Green.paint(format!("{:08X}", offset)));

            for (i, pos) in (offset..(offset + padding)).enumerate() {
                if pos >= src_len {
                    // avoid index out of bounds
                    print!("   ");
                } else if indexes_to_paint.contains(&pos) {
                    // Print the matching bytes colored
                    print!("{} ", Colour::Red.bold().paint(format!("{:02X}", src[pos])));
                } else {
                    print!("{:02X} ", src[pos]);
                }

                if i == 7 {
                    print!(" ");
                }

                std::io::stdout().flush().unwrap();

                if i == 15 {
                    let mut upper_bound = offset + padding;
                    if upper_bound > src_len {
                        // avoid index out of bounds
                        upper_bound = upper_bound - (upper_bound - src_len);
                    }

                    print_ascii_representation(src, offset..upper_bound, &indexes_to_paint);

                    println!();
                }
            }
        }
    }

    pub fn get_all_files_from_paths(paths: Vec<&str>) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for path in paths {
            let filepath = PathBuf::from(path);

            if filepath.is_dir() {
                files.extend(get_all_files_from_dir(filepath));
            } else {
                files.push(filepath);
            }
        }

        files
    }

    pub fn filter_filetypes(files: Vec<PathBuf>, filetypes: &[&str]) -> Vec<PathBuf> {
        files
            .into_iter()
            .filter(|path| {
                filetypes.contains(
                    &path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    fn print_ascii_representation(
        bytes: &[u8],
        indexes_to_print: Range<usize>,
        indexes_to_paint: &Range<usize>,
    ) {
        print!("  |");
        for i in indexes_to_print {
            let ch = ascii_representation(bytes[i]);

            if indexes_to_paint.contains(&i) {
                print!("{}", Colour::Red.bold().paint(format!("{}", ch)));
            } else {
                print!("{}", ch);
            }
        }
        print!("|");

        std::io::stdout().flush().unwrap();
    }

    fn ascii_representation(byte: u8) -> char {
        let ch = byte as char;

        if ch.is_ascii() && !ch.is_ascii_control() {
            ch
        } else {
            '.'
        }
    }

    fn get_all_files_from_dir(dir: PathBuf) -> Vec<PathBuf> {
        let mut filepaths: Vec<PathBuf> = Vec::new();

        visit_dirs(dir, &mut |file| filepaths.push(file)).unwrap();

        filepaths
    }

    fn visit_dirs(dir: PathBuf, cb: &mut dyn FnMut(PathBuf)) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(path, cb)?;
                } else {
                    cb(entry.path());
                }
            }
        }
        Ok(())
    }
}

pub mod search {
    use super::{process, util, Colour, PathBuf};
    use std::fs::File;
    use std::io::Read;

    pub fn search_in_files(pattern: &[u8], files: &[PathBuf]) {
        for filename in files {
            let mut file = match File::open(&filename) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("{}: {}", filename.to_str().unwrap(), err);
                    process::exit(1);
                }
            };

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .expect("Failed reading file to buffer!");

            let result = search_subslice(&buffer, pattern);

            if !result.is_empty() {
                println!("{}", Colour::Purple.paint(filename.to_str().unwrap()));
                util::print_hexdump(result, &buffer, pattern.len());
                println!();
            }
        }
    }

    fn search_subslice(input: &[u8], pattern: &[u8]) -> Vec<usize> {
        let mut match_indexes: Vec<usize> = Vec::new();

        let mut curr_pos_pattern: usize = 0;
        let table_of_ocurrencies = compute_toc(pattern);

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

    fn compute_toc(pattern: &[u8]) -> Vec<usize> {
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
            assert_eq!(vec![3], search_subslice(&text, &[0xFF, 0xFE, 0x00]));
        }

        #[test]
        fn test_string_search() {
            assert_eq!(
                vec![0, 9, 12],
                search_subslice(
                    &[
                        b'A', b'A', b'B', b'A', b'A', b'C', b'A', b'A', b'D', b'A', b'A', b'B',
                        b'A', b'A', b'B', b'A'
                    ],
                    &[b'A', b'A', b'B', b'A']
                )
            )
        }
    }
}
