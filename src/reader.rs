use memmap::Mmap;

pub struct PagedReader {
    /// Starting raw index.
    nl_index: Vec<usize>,
    mmap: Mmap,
}
impl PagedReader {
    pub fn new(mmap: Mmap) -> PagedReader {
        PagedReader {
            nl_index: vec![0 as usize],
            mmap,
        }
    }

    /// find the next "rows" new lines, starting from row_offset position in mmap.
    pub fn find_new_lines(&mut self, rows: u16, row_offset: u64) -> std::io::Result<Vec<usize>> {
        // we need to take `row` lines, starting after `row_offset` lines.
        // since row_offset get increased by row lines, but the count is 0-based, let's handle the special case when row_offset != 0:
        let skip = if row_offset == 0 { 0 } else { row_offset - 1 };
        let end = row_offset as usize + (rows as usize);

        if row_offset as usize + (rows as usize) <= self.nl_index.len() {
            return Ok(self
                .nl_index
                .clone()
                .into_iter()
                .skip(skip as usize)
                .take(rows as usize)
                .collect());
        }
        let last = self.nl_index.last().unwrap().to_owned();
        let missing_indexes = end - self.nl_index.len();

        let res = self.mmap[last..] //start looking from the last found's nl
            .iter()
            .enumerate()
            .filter(|(_i, c)| *c.to_owned() == b"\n"[0])
            .take(missing_indexes)
            .map(|(i, _c)| i + 1 + last as usize); // be sure to readd last, because self.mmap[0] points to `last`.
                                                   // Just extend the slice
        self.nl_index.extend(res);
        let end = std::cmp::min(end, self.nl_index.len());
        Ok(self.nl_index[row_offset as usize..end].to_owned())
    }

    /// rows_to_read = term height
    /// columns_to_read = term width
    pub fn read_file_paged(
        &mut self,
        row_offset: u64,
        _column_offset: u64,
        rows_to_read: u16,
        columns_to_read: u16,
    ) -> std::io::Result<(String, usize)> {
        let indexes = self.find_new_lines(rows_to_read, row_offset)?;
        let indexes_len = indexes.len();
        let mut res = "".to_owned();
        for (i, nl_index) in indexes.iter().enumerate() {
            let start = nl_index.to_owned();
            // If the row length is lesser then the actual terminal width, then use the next \n as limit.
            let row_limit = if i < indexes_len - 1 {
                std::cmp::min(indexes[i + 1], columns_to_read as usize)
            } else {
                columns_to_read as usize
            };
            let end = std::cmp::min(start + row_limit, self.mmap.len());
            let row = self
                .mmap
                .get(start as usize..end as usize)
                .unwrap()
                .to_owned();

            // \t takes more then one char space. Not sure what the correct behaviour should be here.
            let as_string = String::from_utf8(row).unwrap().replace("\t", " ");

            res.push_str(as_string.as_ref());
            if i < indexes_len - 1 {
                res.push_str(&format!("\n\r",));
            }
        }

        Ok((res, indexes_len))
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::PagedReader;
    use memmap::{Mmap, MmapMut};
    use std::io::Write;
    #[test]
    fn test_read_file() {
        let test = b"firsts\nsecond\nthird";
        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mmap = mmap.make_read_only().unwrap();
        let mut paged_reader = PagedReader::new(mmap);
        let expected_rows = 2;
        let (res, rows_red) = paged_reader
            .read_file_paged(0, 0, expected_rows, 1)
            .unwrap();
        let expected = "f\n\rs";
        assert_eq!(expected, res);
        assert_eq!(expected_rows as usize, rows_red)
    }

    #[test]
    fn test_find_new_lines() {
        let test = br#"
abc
"#;
        let expected = vec![0, 1, 5];

        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mut paged_reader = PagedReader::new(mmap.make_read_only().unwrap());
        let res = paged_reader
            .find_new_lines(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);

        let no_newlines = br#""#;
        let expected = vec![0];
        let mut mmap = MmapMut::map_anon(1).expect("Anon mmap");
        (&mut mmap[..]).write(no_newlines).unwrap();
        let mut paged_reader = PagedReader::new(mmap.make_read_only().unwrap());
        let res = paged_reader
            .find_new_lines(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);
    }
}
