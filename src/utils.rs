use ansi_term::Colour;
use std::{io::Write, ops::Range};

use self::search::Matches;

pub mod file;
pub mod search;

#[derive(Debug, Clone)]
pub struct CustomRange {
    range: Range<usize>,
}

impl CustomRange {
    pub fn new(range: Range<usize>) -> Self {
        Self { range }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

impl Eq for CustomRange {}

impl PartialEq for CustomRange {
    fn eq(&self, other: &Self) -> bool {
        self.range.start == other.range.start && self.range.end == other.range.end
    }
}

impl PartialOrd for CustomRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CustomRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.range.start.cmp(&other.range.start)
    }
}

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

pub fn print_hexdump_output(matches: &Matches, bytes_per_line: usize) {
    let mut ascii_repr = Vec::new();

    for range in matches.context_bytes_indexes() {
        let offset = range.range().start;
        print!(
            "{}:  ",
            Colour::Green.paint(format!("{:08X}", offset - (offset % bytes_per_line)))
        );
        for i in range.range() {
            let byte = matches.get_data(i % matches.data_len());

            if matches.indexes().contains(&i) {
                print!("{} ", Colour::Red.bold().paint(format!("{:02X}", byte)));
                ascii_repr.push(format!(
                    "{}",
                    Colour::Red.bold().paint(to_ascii_repr(byte).to_string())
                ));
            } else {
                print!("{:02X} ", byte);
                ascii_repr.push(to_ascii_repr(byte).to_string());
            }

            if bytes_per_line >= 8 && (i + 1) % 8 == 0 {
                print!(" ");
            }

            if (i + 1) % bytes_per_line == 0 {
                print_ascii_repr(&ascii_repr);
                ascii_repr.clear();
            }
        }
    }

    // fix ascii column alignment
    if !ascii_repr.is_empty() {
        let total_chars_in_line = bytes_per_line * 3 + 2;
        let total_chars_bytes_printed = if ascii_repr.len() > 8 {
            ascii_repr.len() * 3 + 1
        } else {
            ascii_repr.len() * 3
        };
        let total_spaces_to_print = total_chars_in_line - total_chars_bytes_printed;

        for _ in 0..total_spaces_to_print {
            print!(" ");
        }
        print_ascii_repr(&ascii_repr);
    }

    std::io::stdout().flush().unwrap();
}

fn print_ascii_repr(ascii_repr: &[String]) {
    print!("|");
    for ascii in ascii_repr {
        print!("{}", ascii);
    }
    println!("|");
}

fn to_ascii_repr(byte: u8) -> char {
    let ch = byte as char;

    if ch.is_ascii() && !ch.is_ascii_control() {
        ch
    } else {
        '.'
    }
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
        assert_eq!(
            strip("\"\"remove only one quote\"\"", '"'),
            "\"remove only one quote\""
        )
    }

    #[test]
    fn test_is_hex() {
        assert_eq!(
            PatternType::from("eeffgg"),
            PatternType::HexStr("eeffgg".to_string())
        )
    }
}
