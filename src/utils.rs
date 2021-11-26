use ansi_term::Colour;
use std::io::Write;

use self::search::Matches;

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

pub fn print_hexdump_output(matches: &Matches, bytes_per_line: usize) {
    let mut offset_iter = matches.offset().iter();
    let mut ascii_repr = Vec::new();

    let mut offset = 0;

    for (i, &byte) in matches.data().iter().enumerate() {
        if (i as i64 - bytes_per_line as i64).abs() % bytes_per_line as i64 == 0 {
            offset = if let Some(&offset) = offset_iter.next() {
                offset
            } else {
                offset + bytes_per_line
            };

            print!(
                "{}:  ",
                Colour::Green.paint(format!("{:08X}", offset - (offset % bytes_per_line)))
            );
        }

        if matches.indexes().iter().any(|indexes| indexes.contains(&i)) {
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

    // fix alignment for ascii column when the data buffer lenght it's not multiple of 16
    if !ascii_repr.is_empty() {
        let remaining = bytes_per_line - ascii_repr.len();
        for _ in 0..remaining {
            print!("   ");
        }
        print!(" ");
        print_ascii_repr(&ascii_repr);
    }

    std::io::stdout().flush().unwrap();
}

fn print_ascii_repr(ascii_repr: &[String]) {
    print!(" |");
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
