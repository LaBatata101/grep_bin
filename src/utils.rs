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

pub fn print_output(matching_indexes: &[Match]) {
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
            _match.data().iter().map(|&c| c as char).collect::<String>()
        );

        std::io::stdout().flush().unwrap();
        println!();
    }
}

pub fn print_hexdump_output(matching_indexes: &[Match], bytes_per_line: usize) {
    for mtch in matching_indexes {
        let mut offset = mtch.offset() - (mtch.offset() % bytes_per_line);
        let mut curr_pos = 0;

        for bytes in mtch.data().chunks(bytes_per_line) {
            let mut ascii_repr = Vec::new();
            print!("{}:  ", Colour::Green.paint(format!("{:08X}", offset)));

            for (i, &byte) in bytes.iter().enumerate() {

                if mtch.index().contains(&curr_pos) {
                    print!("{} ", Colour::Red.bold().paint(format!("{:02X}", byte)));
                    ascii_repr.push(format!("{}", Colour::Red.bold().paint(to_ascii_repr(byte).to_string())));
                } else {
                    print!("{:02X} ", byte);
                    ascii_repr.push(to_ascii_repr(byte).to_string());
                }

                if (i + 1) % 8 == 0 {
                    print!(" ");
                }

                if i == bytes_per_line - 1 {
                    print_ascii_repr(&ascii_repr);
                }

                curr_pos += 1;
            }

            std::io::stdout().flush().unwrap();

            offset += bytes_per_line;
        }
    }

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
