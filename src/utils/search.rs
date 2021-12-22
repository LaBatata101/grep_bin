use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use super::CustomRange;

#[derive(Debug, Clone)]
pub struct Matches {
    indexes: Vec<usize>,
    context_bytes_indexes: BTreeSet<CustomRange>,
    data: Vec<u8>,
    context_bytes_size: usize,
}

impl Matches {
    pub fn new(context_bytes_size: usize) -> Self {
        Self {
            context_bytes_size,
            indexes: Vec::new(),
            context_bytes_indexes: BTreeSet::new(),
            data: Vec::new(),
        }
    }

    pub fn context_bytes_indexes(&self) -> &BTreeSet<CustomRange> {
        &self.context_bytes_indexes
    }

    /// Get a reference to the match's index.
    pub fn indexes(&self) -> &[usize] {
        &self.indexes
    }

    /// Get a reference to the match's data.
    pub fn get_data(&self, index: usize) -> u8 {
        *self.data.get(index).unwrap()
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.indexes.is_empty() && self.indexes.is_empty()
    }

    fn populate_matches(&mut self, indexes: &[usize], buffer: &[u8]) {
        for index in indexes {
            // index where we should start collecting bytes for context
            let offset = index - (index % self.context_bytes_size);

            // Creates the index range for the context bytes.
            let context_bytes_indexes = if offset + self.context_bytes_size <= buffer.len() {
                CustomRange::new(offset..offset + self.context_bytes_size)
            } else {
                CustomRange::new(offset..buffer.len())
            };

            let bytes = &buffer[context_bytes_indexes.range.start..context_bytes_indexes.range.end];
            if self.context_bytes_indexes.insert(context_bytes_indexes) {
                // The actual bytes for context + the matching bytes
                // needed for printing the result
                self.data.extend_from_slice(bytes);
            }
        }

        self.indexes.extend_from_slice(indexes);
    }
}

pub struct Searcher<'a> {
    pattern: &'a [u8],
    matches: Matches,
    context_bytes_size: usize,
    skip_bytes: u64,
}

impl<'a> Searcher<'a> {
    const BUFFER_SIZE: usize = 8192;

    pub fn new(pattern: &'a [u8], context_bytes_size: usize, skip_bytes: u64) -> Self {
        Self {
            pattern,
            matches: Matches::new(context_bytes_size),
            context_bytes_size,
            skip_bytes,
        }
    }

    pub fn search_in_file(&mut self, filepath: &str) -> std::io::Result<()> {
        let mut file = File::open(filepath)?;
        let file_size = file.metadata().unwrap().len() as usize;

        let _pos_in_file = file.seek(SeekFrom::Start(self.skip_bytes)).unwrap_or(0) as usize;
        let mut reader = BufReader::new(file);

        if file_size < self.context_bytes_size {
            self.context_bytes_size = file_size;
        }

        if file_size <= Self::BUFFER_SIZE {
            let mut buffer = Vec::with_capacity(Self::BUFFER_SIZE);
            reader.read_to_end(&mut buffer)?;

            let result = Self::search_slice(&buffer, self.pattern);
            self.matches.populate_matches(&result, &buffer);
        } else {
            let mut buffer = [0; Self::BUFFER_SIZE];
            loop {
                let n = reader.read(&mut buffer).unwrap();

                if n == 0 {
                    break;
                }

                let result = Self::search_slice(&buffer, self.pattern);
                self.matches.populate_matches(&result, &buffer);

                // pos_in_file += Self::BUFFER_SIZE;
            }
        }

        Ok(())
    }

    /// Uses the KMP algorithm to search
    /// Returns a vector of indexes where the slice pattern starts
    pub fn search_slice(src: &[u8], slice: &[u8]) -> Vec<usize> {
        let mut match_indexes: Vec<usize> = Vec::new();

        let mut curr_pos_pattern: usize = 0;
        let table_of_ocurrencies = Self::compute_toc(slice);

        for (i, &ch) in src.iter().enumerate() {
            while curr_pos_pattern > 0 && slice[curr_pos_pattern] != ch {
                curr_pos_pattern = table_of_ocurrencies[curr_pos_pattern - 1];
            }

            if slice[curr_pos_pattern] == ch {
                if curr_pos_pattern == slice.len() - 1 {
                    let pos = i - curr_pos_pattern;
                    match_indexes
                        .extend_from_slice(&(pos..pos + slice.len()).collect::<Vec<usize>>());
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

    /// Get a reference to the searcher's result.
    pub fn result(&self) -> &Matches {
        &self.matches
    }

    /// Return the context bytes size.
    pub fn context_bytes_size(&self) -> usize {
        self.context_bytes_size
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
