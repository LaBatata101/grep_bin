use ansi_term::Colour;
use std::io::Write;

use self::search::Match;

pub mod file;
pub mod search;

#[derive(Debug)]
pub enum PatternType<'a> {
    Str(&'a str),
    HexStr(&'a str),
}

impl<'a> From<&'a str> for PatternType<'a> {
    fn from(pattern: &'a str) -> Self {
        for chr in pattern.chars() {
            if !chr.is_ascii_hexdigit() {
                return PatternType::Str(pattern);
            }
        }

        PatternType::HexStr(pattern)
    }
}

pub fn print_output(matching_indexes: Vec<Match>) {
    for _match in matching_indexes {
        let colored_offset = Colour::Green.paint(format!("{:08X}", _match.index().start));

        let mut hex_bytes = String::new();

        for byte in _match.data() {
            hex_bytes.push_str(&format!("{:02X} ", byte));
        }

        print!(
            "{}:  {}\t\t\t{}",
            colored_offset,
            hex_bytes,
            std::str::from_utf8(_match.data()).unwrap()
        );

        std::io::stdout().flush().unwrap();
        println!();
    }
}