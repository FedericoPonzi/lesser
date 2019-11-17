use memmap::Mmap;

/// find the next "rows" new lines, starting from row_offset position in mmap.
pub fn find_new_lines(mmap: &Mmap, rows: u16, row_offset: u64) -> std::io::Result<Vec<usize>> {
    let initial_line = if row_offset == 0 && mmap.len() > 0 {
        vec![0 as usize]
    } else {
        vec![]
    };
    let res = mmap
        .iter()
        .enumerate()
        .filter(|(_i, c)| *c.to_owned() == b"\n"[0])
        .skip(row_offset as usize)
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
) -> std::io::Result<String> {
    let indexes = find_new_lines(&memmap, rows_to_read, row_offset)?;
    let indexes_len = indexes.len();
    let mut res = "".to_owned();
    for (i, nl_index) in indexes.iter().enumerate() {
        let start = nl_index.to_owned();
        let row_limit = if i < indexes.len() - 1 {
            std::cmp::min(indexes[i + 1], columns_to_read as usize)
        } else {
            columns_to_read as usize
        };
        let end = start + row_limit;
        /*res.push_str(&format!(
            "Read: row: {}/{},  {} - {}, cols: {} -----------------------------------\r\n",
            i, row_offset, start, end, columns_to_read
        ));*/
        let row = memmap.get(start as usize..end as usize).unwrap().to_owned();
        let as_string = String::from_utf8(row).unwrap();
        res.push_str(as_string.as_ref());
        if i < indexes_len - 1 {
            res.push_str("\n\r");
        }
    }

    Ok(res)
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
        let res = read_file_paged(&mmap, 0, 0, 2, 1).unwrap();
        let expected = "f\n\rs";
        assert_eq!(expected, res);
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
