use memmap::Mmap;
use std::{cmp, io};

type StartIndex = usize;
type EndIndex = usize;

pub struct PagedReader {
    /// Start-end row indexes. A row is delimited by an EOL char.
    /// This vector referes to the file, so it's independent from the screen-size.
    rows_indexes: Vec<(StartIndex, EndIndex)>,
    mmap: Mmap,
}

impl PagedReader {
    pub fn new(mmap: Mmap) -> PagedReader {
        PagedReader {
            rows_indexes: vec![],
            mmap,
        }
    }

    /// rows_to_read = term height
    /// columns_to_read = term width
    /// Returns a page. Will start reading from row_offset / column offset and will read
    /// rows_to_read rows, and columns_to_read columns.
    pub fn read_file_paged(
        &mut self,
        row_offset: u64,
        column_offset: u64,
        rows_to_read: u16,
        columns_to_read: u16,
    ) -> std::io::Result<(String, usize, usize)> {
        let indexes = self.get_rows_indexes(rows_to_read, row_offset)?;
        let indexes_len = indexes.len();
        let mut res = String::new();
        let mut has_text = false;
        for (i, (start_row, end_row)) in indexes.into_iter().enumerate() {
            let current_column = start_row + column_offset as usize;
            let start = cmp::min(current_column, end_row);
            let end = cmp::min(end_row, current_column + columns_to_read as usize);

            let row = &self.mmap[start..end];
            let as_string = String::from_utf8_lossy(row);

            has_text = has_text || !as_string.is_empty();

            res.push_str(as_string.as_ref());
            if i < indexes_len - 1 {
                res.push_str("\n\r");
            }
        }
        // If horizontal scrolling hasn't returned any char, then won't scroll.
        let cols_red = if has_text {
            columns_to_read as usize
        } else {
            0
        };
        //TODO: indexes_len = rows_red
        Ok((res, indexes_len, cols_red))
    }

    /// find the next "rows" new lines, starting from row_offset position in self.mmap.
    fn get_rows_indexes(
        &mut self,
        rows: u16,
        row_offset: u64,
    ) -> io::Result<Vec<(StartIndex, EndIndex)>> {
        // we need to take `row` lines, starting after `row_offset` lines.
        // since row_offset get increased by row lines, but the count is 0-based, let's handle the special case when row_offset != 0:

        let default = row_offset.saturating_sub(rows as u64) as usize;
        //let default = cmp::max(0, row_offset as i64 - (rows as i64)) as usize;
        let to_row = (row_offset as usize)
            .checked_add(rows as usize)
            .unwrap_or(default as usize);

        let file_is_all_read = self
            .rows_indexes
            .last()
            .map(|(_start, end)| {
                // if the file is empty. mmap is at least 1. But if the file is non-empty, then end and mmap.len() should match.
                *end >= self.mmap.len() - 1
            })
            .unwrap_or(false);

        let indexes_are_known = to_row > self.rows_indexes.len();
        if !file_is_all_read && indexes_are_known {
            self.fetch_missing_rows_indexes(to_row);
        }

        let skip_offset = cmp::min(self.rows_indexes.len(), row_offset as usize).saturating_sub(1);
        Ok(self
            .rows_indexes
            .iter()
            .skip(skip_offset)
            .cloned()
            .take(rows as usize)
            .collect())
    }
    fn fetch_missing_rows_indexes(&mut self, to_row: usize) {
        let last_found = self
            .rows_indexes
            .last()
            .map(|(_start, end)| end + 1) // end is the newline char, we need to start looking after it.
            .unwrap_or(0)
            .clone();

        let missing_indexes = to_row - self.rows_indexes.len();

        // Left side, is inclusive.
        let mut last = last_found;

        let mut limit = missing_indexes.checked_mul(2).unwrap_or(usize::MAX);

        const NEW_LINE_CHAR: u8 = b'\n';

        let mmap_len = self.mmap.len();
        // -1 because mmap is 1 even if the file is empty.
        let is_last_char = |i| i == mmap_len - 1;

        for (i, c) in self.mmap[last_found..] // start looking from the lastly found nl
            .iter()
            .enumerate()
        {
            if *c == NEW_LINE_CHAR {
                let found = i + last_found;
                self.rows_indexes.push((last, found as usize));
                last = found + 1 as usize;
                limit -= 1;

                // If I've searched for enough indexes, let's defer the search of other nl for later
                if limit <= 0 {
                    break;
                }
            } else if is_last_char(i) {
                self.rows_indexes.push((last, self.mmap.len()));
            }
        }
    }

    pub fn cached_rows(&self) -> usize {
        self.rows_indexes.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::lesser::reader::PagedReader;
    use memmap::MmapMut;
    use std::io::Write;

    #[test]
    fn test_read_file_columned() {
        let test = b"firsts\nsecond\nthird";
        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mmap = mmap.make_read_only().unwrap();
        let mut paged_reader = PagedReader::new(mmap);
        let expected_rows = 2;
        let (res, rows_red, cols_red) = paged_reader
            .read_file_paged(0, 0, expected_rows, 1)
            .unwrap();
        let expected = "f\n\rs";
        assert_eq!(expected, res);
        assert_eq!(expected_rows as usize, rows_red);
        assert_eq!(1, cols_red);
    }

    #[test]
    fn test_read_half_file() {
        let test = b"firsts\nsecond\nthird";
        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mmap = mmap.make_read_only().unwrap();
        let mut paged_reader = PagedReader::new(mmap);
        let expected_rows = 2;
        let (res, rows_red, cols_red) = paged_reader
            .read_file_paged(0, 0, expected_rows, 10)
            .unwrap();
        let expected = "firsts\n\rsecond";
        assert_eq!(expected, res);
        assert_eq!(expected_rows as usize, rows_red);
        assert_eq!(10, cols_red);
    }

    #[test]
    fn test_read_whole_file() {
        let test = b"firsts\nsecond\nthird";
        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mmap = mmap.make_read_only().unwrap();
        let mut paged_reader = PagedReader::new(mmap);
        let expected_rows = 3;
        let (res, rows_red, cols_red) = paged_reader
            .read_file_paged(0, 0, expected_rows, 10)
            .unwrap();
        let expected = String::from_utf8_lossy(test).replace("\n", "\n\r");
        assert_eq!(expected, res);
        assert_eq!(expected_rows as usize, rows_red);
        assert_eq!(10, cols_red);
    }

    #[test]
    fn test_find_new_lines() {
        let test = br#"
abc"#;
        let expected = vec![(0, 0), (1, 4)];

        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mut paged_reader = PagedReader::new(mmap.make_read_only().unwrap());
        let res = paged_reader
            .get_rows_indexes(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);

        let no_newlines = br#""#;
        let expected = vec![(0, 1)];
        let mut mmap = MmapMut::map_anon(1).expect("Anon mmap");
        (&mut mmap[..]).write(no_newlines).unwrap();
        let mut paged_reader = PagedReader::new(mmap.make_read_only().unwrap());
        let res = paged_reader
            .get_rows_indexes(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);
    }
}
