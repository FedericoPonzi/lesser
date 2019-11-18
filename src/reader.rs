use memmap::Mmap;

/// find the next "rows" new lines, starting from row_offset position in mmap.
// TODO: Should cache the indexes.
pub fn find_new_lines(mmap: &Mmap, rows: u16, row_offset: u64) -> std::io::Result<Vec<usize>> {
    let initial_line = if row_offset == 0 && mmap.len() > 0 {
        vec![0 as usize]
    } else {
        vec![]
    };
    // we need to take `row` lines, starting after `row_offset` lines.
    // since row_offset get increased by row lines, but the count is 0-based, let's handle the special case when row_offset != 0:
    let skip = if row_offset == 0 { 0 } else { row_offset - 1 };

    let res = mmap
        .iter()
        .enumerate()
        .filter(|(_i, c)| *c.to_owned() == b"\n"[0])
        .skip(skip as usize)
        .take(rows as usize)
        .map(|(i, _c)| i + 1 as usize);
    Ok(initial_line
        .into_iter()
        .chain(res)
        .take(rows as usize)
        .collect())
}

/// rows_to_read = term height
/// columns_to_read = term width
pub fn read_file_paged(
    memmap: &Mmap,
    row_offset: u64,
    _column_offset: u64,
    rows_to_read: u16,
    columns_to_read: u16,
) -> std::io::Result<(String, usize)> {
    let indexes = find_new_lines(&memmap, rows_to_read, row_offset)?;
    let indexes_len = indexes.len();
    let mut res = "".to_owned();
    for (i, nl_index) in indexes.iter().enumerate() {
        let start = nl_index.to_owned();
        let row_limit = if i < indexes_len - 1 {
            std::cmp::min(indexes[i + 1], columns_to_read as usize)
        } else {
            columns_to_read as usize
        };
        let end = std::cmp::min(start + row_limit, memmap.len());
        let row = memmap.get(start as usize..end as usize).unwrap().to_owned();
        let as_string = String::from_utf8(row).unwrap().replace("\t", " "); // \t takes more then one char space. Not sure what the correct behaviour should be here.
        res.push_str(as_string.as_ref());
        if i < indexes_len - 1 {
            res.push_str(&format!("\n\r",));
        }
    }

    Ok((res, indexes_len))
}

#[cfg(test)]
mod tests {
    use crate::reader::{find_new_lines, read_file_paged};
    use memmap::{Mmap, MmapMut};
    use std::io::Write;
    #[test]
    fn test_read_file() {
        let test = b"firsts\nsecond\nthird";
        let mut mmap = MmapMut::map_anon(test.len()).expect("Anon mmap");
        (&mut mmap[..]).write(test).unwrap();
        let mmap = mmap.make_read_only().unwrap();
        let expected_rows = 2;
        let (res, rows_red) = read_file_paged(&mmap, 0, 0, expected_rows, 1).unwrap();
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
        let res =
            find_new_lines(&mmap.make_read_only().unwrap(), 10, 0).expect("No newlines found.");
        assert_eq!(res, expected);

        let no_newlines = br#""#;
        let expected = vec![0];
        let mut mmap = MmapMut::map_anon(1).expect("Anon mmap");
        (&mut mmap[..]).write(no_newlines).unwrap();
        let res =
            find_new_lines(&mmap.make_read_only().unwrap(), 10, 0).expect("No newlines found.");
        assert_eq!(res, expected);
    }
}
