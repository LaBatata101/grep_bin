use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;

#[derive(Debug)]
pub struct Match {
    index: Range<usize>,
    data: Vec<u8>,
    offset: usize,
}

impl Match {
    pub fn new(offset: usize, index: Range<usize>, data: Vec<u8>) -> Self {
        Self {
            index,
            data,
            offset,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get a reference to the match's index.
    pub fn index(&self) -> Range<usize> {
        self.index.clone()
    }

    /// Get a reference to the match's data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub struct Searcher<'a> {
    pattern: &'a [u8],
    result: Vec<Vec<Match>>,
    context_bytes_size: usize,
}

impl<'a> Searcher<'a> {
    const BUFFER_SIZE: usize = 8192;

    pub fn new(pattern: &'a [u8], context_bytes_size: usize) -> Self {
        Self {
            pattern,
            result: Vec::new(),
            context_bytes_size,
        }
    }

    pub fn search_in_file(&mut self, filepath: &str) -> std::io::Result<()> {
        let file = File::open(filepath)?;

        let mut reader = BufReader::new(file);
        let mut buffer = [0; Self::BUFFER_SIZE];
        let mut pos_in_file = 0;

        loop {
            let n = reader.read(&mut buffer).unwrap();

            if n == 0 {
                break;
            }

            let result = Self::search_slice(&buffer, self.pattern);

            if !result.is_empty() {
                // Convert the vector of indexes that match the pattern into Match objects
                let result = result
                    .iter()
                    .map(|&index| {
                        // index where we should start collecting bytes for context
                        let offset = index - (index % self.context_bytes_size);

                        // search_for_slice only return the index where the match start so we need to
                        // create a range with all indexes from the match
                        let match_indexes = index..index + self.pattern.len();

                        // Creates the index range for the context bytes.
                        // this can contain all the indexes for the match or partially, depends on
                        // context_bytes_size and the pattern size
                        let mut context_bytes_indexes = offset..offset + self.context_bytes_size;

                        // In case context_bytes_size doesn't contain all of the match indexes we
                        // need to extend the end of the range
                        if context_bytes_indexes.end < match_indexes.end {
                            context_bytes_indexes.end += self.context_bytes_size;
                        }

                        // The actual bytes for context + the matching bytes
                        // only for printing the result
                        let context_bytes = buffer[context_bytes_indexes].to_vec();

                        // Now we need to know the indexes of the match inside of context_bytes
                        let mut match_indexes = match_indexes.start % context_bytes.len()
                            ..match_indexes.end % context_bytes.len();

                        if match_indexes.end == 0 {
                            match_indexes.end = self.context_bytes_size;
                        }

                        // The index is relative to the position in the current buffer we are
                        // reading from the file, but we need to store the position relative to the
                        // whole file
                        Match::new(index + pos_in_file, match_indexes, context_bytes)
                    })
                    .collect();

                self.result.push(result);
            }
            pos_in_file += Self::BUFFER_SIZE;
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

    /// Get a reference to the searcher's result.
    pub fn result(&self) -> &[Vec<Match>] {
        &self.result
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
