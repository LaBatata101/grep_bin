use ansi_term::Colour;
use std::{io::Write, ops::Range};

use self::search::Match;

pub mod file;
pub mod search;

#[derive(Debug, PartialEq)]
pub enum PatternType {
    Str(String),
    HexStr(String),
}

impl<'a> From<&'a str> for PatternType {
    fn from(pattern: &'a str) -> Self {
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let quote_striped = strip(pattern, '"');

            if quote_striped.starts_with("\\\"") && quote_striped.ends_with("\\\"") {
                return PatternType::Str(quote_striped.chars().filter(|&c| c != '\\').collect());
            }
            PatternType::Str(quote_striped.to_string())
        } else {
            PatternType::HexStr(pattern.to_string())
        }
    }
}

fn strip(src: &str, p: char) -> &str {
    if let Some(prefix_striped) = src.strip_prefix(p) {
        if let Some(suffix_striped) = prefix_striped.strip_suffix(p) {
            suffix_striped
        } else {
            prefix_striped
        }
    } else {
        src
    }
}

pub fn print_hexdump_output(matches: &[Match], context_bytes_size: usize) {
    for _match in matches {
        print!("{}:  ", Colour::Green.paint(format!("{:08X}", _match.offset)));

        print_bytes_as_hex_char_colored(&_match.bytes, &_match.indexes_to_paint, Colour::Red, context_bytes_size);

        let total_bytes_to_print = _match.bytes.len();

        // When we have two matches (each match is one line that is going to be printed) and one of these matches
        // total_bytes_to_print is less than context_bytes_size we need to fill with empty spaces to align the
        // ascii column with the ascii column from the previous printed line.
        if matches.len() >= 2 && total_bytes_to_print < context_bytes_size {
            let total_chars_in_line = context_bytes_size * 3 + 2;
            let total_spaces_to_print = total_chars_in_line - total_bytes_to_print * 3;

            for _ in 0..total_spaces_to_print {
                print!(" ");
            }
        }

        let colored_ascii_repr = bytes_to_ascii_colored_repr(&_match.bytes, &_match.indexes_to_paint, Colour::Red);
        println!("|{colored_ascii_repr}|");
    }
    std::io::stdout().flush().unwrap();
}

fn print_bytes_as_hex_char_colored(bytes: &[u8], indexes_to_paint: &[Range<usize>], color: Colour, blocks_size: usize) {
    for (i, byte) in bytes.iter().enumerate() {
        if indexes_to_paint.iter().any(|range| range.contains(&i)) {
            print!("{} ", color.bold().paint(format!("{:02X}", byte)));
        } else {
            print!("{:02X} ", byte);
        }

        if blocks_size >= 8 && (i + 1) % 8 == 0 {
            print!(" ");
        }
    }
}

fn bytes_to_ascii_colored_repr(bytes: &[u8], indexes_to_paint: &[Range<usize>], color: Colour) -> String {
    bytes
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            let ch = byte as char;
            // NOTE: Trye to find a better way of doing this
            if indexes_to_paint.iter().any(|range| range.contains(&i)) {
                if ch.is_ascii() && !ch.is_ascii_control() {
                    color.bold().paint(ch.to_string()).to_string()
                } else {
                    color.bold().paint(".").to_string()
                }
            } else if ch.is_ascii() && !ch.is_ascii_control() {
                ch.to_string()
            } else {
                String::from(".")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_backslash() {
        assert_eq!(
            PatternType::from("\"backslash\""),
            PatternType::Str("backslash".to_string())
        )
    }

    #[test]
    fn test_trim_backslash_with_quotes() {
        assert_eq!(
            PatternType::from("\"\\\"backslash with quote\\\"\""),
            PatternType::Str("\"backslash with quote\"".to_string())
        )
    }

    #[test]
    fn test_strip() {
        assert_eq!(strip("\"\"remove only one quote\"\"", '"'), "\"remove only one quote\"")
    }

    #[test]
    fn test_is_hex() {
        assert_eq!(PatternType::from("eeffgg"), PatternType::HexStr("eeffgg".to_string()))
    }
}
