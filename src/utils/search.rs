use std::collections::BTreeMap;
use std::ops::Range;


use super::file::read_file_and;

#[derive(Debug, Clone)]
pub struct Match {
    pub offset: usize,
    pub indexes_to_paint: Vec<Range<usize>>,
    pub bytes: Vec<u8>,
}

pub struct Searcher<'a> {
    pattern: &'a [u8],
    context_bytes_size: usize,
    skip_bytes: u64,
}

impl<'a> Searcher<'a> {
    pub fn new(pattern: &'a [u8], context_bytes_size: usize, skip_bytes: u64) -> Self {
        Self {
            pattern,
            context_bytes_size,
            skip_bytes,
        }
    }



    pub fn search_in_file(&mut self, filepath: &str) -> std::io::Result<Vec<Match>> {
        let mut matches = Vec::new();
        read_file_and(filepath, self.skip_bytes, |buffer, pos_in_file| {
            matches.extend_from_slice(&self.search(buffer, pos_in_file));
        })?;

        Ok(matches)
    }

    fn search(&self, buffer: &[u8], pos_in_file: usize) -> Vec<Match> {
        let match_indexes = Self::search_slice(buffer, self.pattern);

        if match_indexes.is_empty() {
            return Vec::new();
        }

        let mut offset_indexes: BTreeMap<usize, Vec<Range<usize>>> = BTreeMap::new();

        for index in match_indexes {
            let offset = index.start - (index.start % self.context_bytes_size);
            let indexes = offset_indexes.entry(offset).or_insert(Vec::new());

            if index.end - offset > self.context_bytes_size {
                indexes.push(index.start - offset..self.context_bytes_size);

                let new_offset = offset + self.context_bytes_size;

                let indexes = offset_indexes.entry(new_offset).or_insert(Vec::new());
                indexes.push(0..index.end - new_offset);
            } else {
                indexes.push(index.start - offset..index.end - offset);
            }
        }

        offset_indexes
            .iter()
            .map(|(&offset, indexes)| {
                let bytes = if offset + self.context_bytes_size <= buffer.len() {
                    buffer[offset..offset + self.context_bytes_size].to_vec()
                } else {
                    buffer[offset..].to_vec()
                };

                Match {
                    offset: offset + pos_in_file,
                    indexes_to_paint: indexes.clone(),
                    bytes,
                }
            })
            .collect()
    }

    /// Uses the KMP algorithm to search
    /// Returns a vector of indexes where the slice pattern starts
    pub fn search_slice(src: &[u8], slice: &[u8]) -> Vec<Range<usize>> {
        let mut match_indexes = Vec::new();

        let mut curr_pos_pattern: usize = 0;
        let table_of_ocurrencies = Self::compute_toc(slice);

        for (i, &ch) in src.iter().enumerate() {
            while curr_pos_pattern > 0 && slice[curr_pos_pattern] != ch {
                curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern - 1];
            }

            if slice[curr_pos_pattern] == ch {
                if curr_pos_pattern == slice.len() - 1 {
                    let pos = i - curr_pos_pattern;
                    match_indexes.push(pos..pos + slice.len());
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
}

/* #[cfg(test)]
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
} */
