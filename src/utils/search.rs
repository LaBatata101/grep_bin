use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;

#[derive(Debug)]
pub struct Match<'a> {
    index: Range<usize>,
    data: &'a [u8],
}

impl<'a> Match<'a> {
    pub fn new(index: Range<usize>, data: &'a [u8]) -> Self {
        Self { index, data }
    }

    /// Get a reference to the match's index.
    pub fn index(&self) -> &Range<usize> {
        &self.index
    }

    /// Get a reference to the match's data.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

const BUFFER_SIZE: usize = 8192;

pub fn search_in_file(pattern: &[u8], mut file: BufReader<File>) -> Vec<Vec<Match>> {
    let mut buffer = [0; BUFFER_SIZE];
    let mut matches = Vec::new();
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }

        let result = search_subslice(&buffer, pattern);

        if !result.is_empty() {
            matches.push(result);
        }
    }

    matches
}

fn search_subslice<'a>(input: &[u8], slice: &'a [u8]) -> Vec<Match<'a>> {
    let mut match_indexes: Vec<usize> = Vec::new();

    let mut curr_pos_pattern: usize = 0;
    let table_of_ocurrencies = compute_toc(slice);

    for (i, &ch) in input.iter().enumerate() {
        while curr_pos_pattern > 0 && slice[curr_pos_pattern] != ch {
            curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern - 1];
        }

        if slice[curr_pos_pattern] == ch {
            if curr_pos_pattern == slice.len() - 1 {
                match_indexes.push(i - curr_pos_pattern);
                curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern];
            } else {
                curr_pos_pattern += 1;
            }
        }
    }

    match_indexes
        .iter()
        .map(|&index| {
            if slice.len() > 1 {
                Match::new(index..index + slice.len(), slice)
            } else {
                Match::new(index..index, slice)
            }
        })
        .collect()
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
        // assert_eq!(vec![3..6], search_subslice(&text, &[0xFF, 0xFE, 0x00]));
    }

    #[test]
    fn test_string_search() {
        // assert_eq!(
        //     vec![0..4, 9..13, 12..16],
        //     search_subslice(
        //         &[
        //             b'A', b'A', b'B', b'A', b'A', b'C', b'A', b'A', b'D', b'A', b'A', b'B', b'A',
        //             b'A', b'B', b'A'
        //         ],
        //         &[b'A', b'A', b'B', b'A']
        //     )
        // )
    }
}
