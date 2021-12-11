use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Matches {
    indexes: Vec<Range<usize>>,
    data: Vec<u8>,
    offset: Vec<usize>,
    pattern_len: usize,
    context_bytes_size: usize,
    curr_context_bytes_indexes: Option<Range<usize>>,
}

impl Matches {
    pub fn new(pattern_len: usize, context_bytes_size: usize) -> Self {
        Self {
            pattern_len,
            context_bytes_size,
            indexes: Vec::new(),
            data: Vec::new(),
            offset: Vec::new(),
            curr_context_bytes_indexes: None,
        }
    }

    pub fn offset(&self) -> &[usize] {
        &self.offset
    }

    /// Get a reference to the match's index.
    pub fn indexes(&self) -> &[Range<usize>] {
        &self.indexes
    }

    /// Get a reference to the match's data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.indexes.is_empty() && self.indexes.is_empty()
    }

    fn populate_matches(&mut self, index: usize, pos_in_file: usize, buffer: &[u8]) {
        // index where we should start collecting bytes for context
        let offset = index - (index % self.context_bytes_size);

        // search_for_slice only return the index where the match start so we need to
        // create a range with all indexes from the match
        let match_indexes = index..index + self.pattern_len;

        // Creates the index range for the context bytes.
        // context_bytes_size_indexes can contain all the indexes for the match or partially, depends on
        // context_bytes_size and the pattern size
        let mut context_bytes_indexes = if offset + self.context_bytes_size <= buffer.len() {
            offset..offset + self.context_bytes_size
        } else {
            offset..buffer.len()
        };

        // In case context_bytes_size doesn't contain all of the match indexes we
        // need to extend the end of the range
        if context_bytes_indexes.end < match_indexes.end {
            context_bytes_indexes.end += self.context_bytes_size;
        }

        let context_bytes_indexes = Some(context_bytes_indexes);
        if context_bytes_indexes != self.curr_context_bytes_indexes { // Check if context_bytes_indexes was already added
            self.curr_context_bytes_indexes = context_bytes_indexes.clone();

            // The actual bytes for context + the matching bytes
            // needed for printing the result
            self.data.extend_from_slice(&buffer[context_bytes_indexes.unwrap()]);

            // The index is relative to the position in the current buffer we are
            // reading from the file, but we need to store the position relative to the
            // whole file
            self.offset.push(index + pos_in_file);
        }

        // Now we need to know the indexes of the match inside of context_bytes
        let mut match_indexes = match_indexes.start % self.data.len()
            ..match_indexes.end % self.data.len();

        if match_indexes.end < match_indexes.start {
            match_indexes.end = match_indexes.start + self.pattern_len;
        }

        self.indexes.push(match_indexes);
    }
}

pub struct Searcher<'a> {
    pattern: &'a [u8],
    matches: Matches,
    context_bytes_size: usize,
}

impl<'a> Searcher<'a> {
    const BUFFER_SIZE: usize = 8192;

    pub fn new(pattern: &'a [u8], context_bytes_size: usize) -> Self {
        Self {
            pattern,
            matches: Matches::new(pattern.len(), context_bytes_size),
            context_bytes_size,
        }
    }

    pub fn search_in_file(&mut self, filepath: &str) -> std::io::Result<()> {
        let file = File::open(filepath)?;
        let file_size = file.metadata().unwrap().len() as usize;

        let mut reader = BufReader::new(file);
        let mut pos_in_file = 0;

        if file_size < self.context_bytes_size {
            self.context_bytes_size = file_size;
        }

        if file_size <= Self::BUFFER_SIZE {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;

            let result = Self::search_slice(&buffer, self.pattern);
            for index in result {
                self.matches.populate_matches(index, 0, &buffer);
            }
        } else {
            let mut buffer = [0; Self::BUFFER_SIZE];
            loop {
                let n = reader.read(&mut buffer).unwrap();

                if n == 0 {
                    break;
                }

                let result = Self::search_slice(&buffer, self.pattern);
                for index in result {
                    self.matches.populate_matches(index, pos_in_file, &buffer);
                }
                pos_in_file += Self::BUFFER_SIZE;
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
