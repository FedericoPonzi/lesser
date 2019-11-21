use memmap::Mmap;

pub struct PagedReader {
    /// Starting raw index.
    eol_indexes: Vec<(usize, usize)>,
    mmap: Mmap,
}
impl PagedReader {
    pub fn new(mmap: Mmap) -> PagedReader {
        PagedReader {
            eol_indexes: vec![],
            mmap,
        }
    }
    ///TODO: correctly handle newlines only file.
    /// find the next "rows" new lines, starting from row_offset position in mmap.
    pub fn find_new_lines(
        &mut self,
        rows: u16,
        row_offset: u64,
    ) -> std::io::Result<Vec<(usize, usize)>> {
        // we need to take `row` lines, starting after `row_offset` lines.
        // since row_offset get increased by row lines, but the count is 0-based, let's handle the special case when row_offset != 0:
        let end = row_offset as usize + (rows as usize);

        if self.eol_indexes.len() > 0 && self.eol_indexes.last().unwrap().1 == self.mmap.len() - 1
            || row_offset as usize + (rows as usize) <= self.eol_indexes.len()
        {
            return Ok(self
                .eol_indexes
                .clone()
                .into_iter()
                .skip(row_offset as usize)
                .take(rows as usize)
                .collect());
        }
        let last_found = self
            .eol_indexes
            .last()
            .map(|(_start, finish)| finish + 1) // finish is the newline char, we need to start looking after it.
            .unwrap_or(0)
            .to_owned();
        let missing_indexes = end - self.eol_indexes.len();

        let mut res = vec![];
        // Left side, is inclusive.
        let mut last = last_found;

        let nl = b"\n"[0];
        for (i, c) in self.mmap[last_found..] //start looking from the lastly found nl
            .iter()
            .enumerate()
        {
            if *c == nl {
                let found = i + last_found;
                res.push((last, found as usize));
                last = found + 1 as usize;
                if res.len() == missing_indexes {
                    break;
                }
            } else if i == self.mmap.len() - 1 {
                if i != 0 {
                    res.push((last, self.mmap.len()));
                }
            }
        }
        self.eol_indexes.extend(res);

        //TODO: self.find_new_lines(rows, row_offset);
        let end = std::cmp::min(end, self.eol_indexes.len());
        Ok(self.eol_indexes[row_offset as usize..end].to_owned())
    }

    /// rows_to_read = term height
    /// columns_to_read = term width
    pub fn read_file_paged(
        &mut self,
        row_offset: u64,
        column_offset: u64,
        rows_to_read: u16,
        columns_to_read: u16,
    ) -> std::io::Result<(String, usize, usize)> {
        let indexes = self.find_new_lines(rows_to_read, row_offset)?;
        let indexes_len = indexes.len();
        let mut res = "".to_owned();
        let mut has_text = false;
        for (i, nl_index) in indexes.iter().enumerate() {
            let (start_row, end_row) = nl_index.to_owned();

            let end = std::cmp::min(
                end_row,
                start_row + column_offset as usize + columns_to_read as usize,
            )
            .to_owned();

            let start = std::cmp::min(start_row + column_offset as usize, end);

            let row = &self.mmap[start as usize..end as usize];

            //res.push_str(format!("start:{}, end:{}", start_row, end_row).as_ref());
            // \t takes more then one char space. Not sure what the correct behaviour should be here.
            let as_string = String::from_utf8_lossy(row).to_string().replace("\t", " ");
            if as_string.len() > 0 {
                has_text = true;
            }
            res.push_str(as_string.as_ref());
            if i < indexes_len - 1 {
                res.push_str(&format!("\n\r",));
            }
        }
        // If horizontal scrolling hasn't returned any char, then won't scroll.
        let cols_red = if has_text {
            columns_to_read as usize
        } else {
            0
        };
        Ok((res, indexes_len, cols_red))
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
        let (res, rows_red, cols_red) = paged_reader
            .read_file_paged(0, 0, expected_rows, 1)
            .unwrap();
        let expected = "f\n\rs";
        assert_eq!(expected, res);
        assert_eq!(expected_rows as usize, rows_red);
        assert_eq!(1, cols_red);
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
            .find_new_lines(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);

        let no_newlines = br#""#;
        let expected = vec![];
        let mut mmap = MmapMut::map_anon(1).expect("Anon mmap");
        (&mut mmap[..]).write(no_newlines).unwrap();
        let mut paged_reader = PagedReader::new(mmap.make_read_only().unwrap());
        let res = paged_reader
            .find_new_lines(10, 0)
            .expect("No newlines found.");
        assert_eq!(res, expected);
    }
}
