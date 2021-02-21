use crate::lesser::reader::PagedReader;
use log::debug;
use std::cmp::min;
use std::io::Result;

type PageToPrint = Option<String>;

pub struct ScreenMoveHandler {
    /// Row-wise position currently displayed
    row_offset: u64,
    /// Column-wise position currently displayed
    col_offset: u64,
    paged_reader: PagedReader,
}

impl ScreenMoveHandler {
    pub(crate) fn new(paged_reader: PagedReader) -> Self {
        ScreenMoveHandler {
            row_offset: 0,
            col_offset: 0,
            paged_reader,
        }
    }
    /// The first page
    pub(crate) fn initial_screen(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        let (page, rows_red, cols_red) = self.paged_reader.read_file_paged(0, 0, rows, cols)?;
        self.row_offset += rows_red as u64;
        self.col_offset += cols_red as u64;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    /// Doesn't trigger any movement, just rereads the current screen.
    pub(crate) fn reload(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        // reset the index back to the start of the line:
        self.col_offset = 0;
        // Re read this page:
        let min_row_offset = (self.row_offset as i64) - (rows as i64);
        self.row_offset = std::cmp::max(min_row_offset, 0) as u64;

        let (page, rows_red, cols_red) =
            self.paged_reader
                .read_file_paged(self.row_offset, self.col_offset, rows, cols)?;
        self.row_offset += rows_red as u64;
        self.col_offset += cols_red as u64;

        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    // X axis: read the page and moves the col offset position
    // from self.col_offset to self.col_offset+ cols_red.
    fn move_x(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        // Re read the same rows
        let fixed_row_offset = std::cmp::max((self.row_offset as i64) - (rows as i64), 0) as u64;

        let (page, _rows_red, cols_red) =
            self.paged_reader
                .read_file_paged(fixed_row_offset, self.col_offset, rows, cols)?;
        self.col_offset += cols_red as u64;
        let ret = if cols_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    /// Move left one column
    pub(crate) fn move_left(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move right request");
        // I need to read not from the beginning of this page, but from the beginning of the last page. Thus * 2.
        let min_col_offset = (self.col_offset as i64) - ((cols + 1) as i64);
        // we're not moving by rows:
        self.col_offset = std::cmp::max(min_col_offset, 0) as u64;
        self.move_x(rows, cols)
    }

    /// Move right one column
    pub(crate) fn move_right(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move right request");
        // This is used to avoid going back one screen if the move_x has returned None
        // (e.g it hasn't read anything).
        let old_offset = self.col_offset;
        self.col_offset = (self.col_offset as i64 - (cols - 1) as i64) as u64; // - cols as i64) as u64;
        let ret = self.move_x(rows, cols);
        ret.iter().for_each(|opt| {
            if opt.is_none() && old_offset != self.col_offset {
                self.col_offset = old_offset;
            }
        });
        ret
    }

    // Y axis:

    fn move_y(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        let fixed_col_offset = std::cmp::max((self.col_offset as i64) - (cols as i64), 0) as u64;
        let (page, rows_red, _cols_red) =
            self.paged_reader
                .read_file_paged(self.row_offset, fixed_col_offset, rows, cols)?;
        // fix offset
        self.row_offset = min(self.row_offset, self.paged_reader.cached_rows() as u64);
        self.row_offset += rows_red as u64;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    pub(crate) fn move_down_page(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move down page request");
        self.move_y(rows, cols)
    }
    pub(crate) fn move_up_page(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move up request");

        // I need to read not from the beginning of this page, but from the beginning of the last page. Thus * 2.
        let min_row_offset = (self.row_offset as i64) - (rows as i64) * 2;
        self.row_offset = std::cmp::max(min_row_offset, 0) as u64;
        self.move_y(rows, cols)
    }
    pub(crate) fn move_up(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move up request");
        let min_row_offset = (self.row_offset as i64) - ((rows + 1) as i64);
        self.row_offset = std::cmp::max(min_row_offset, 0) as u64;
        self.move_y(rows, cols)
    }

    pub(crate) fn move_down(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move up request");
        // This is used to avoid going back one screen if the move_x has returned None
        // (e.g it hasn't read anything).
        let old_offset = self.row_offset;
        self.row_offset = (self.row_offset as i64 - (rows - 1) as i64) as u64; // - cols as i64) as u64;
        let ret = self.move_y(rows, cols);
        ret.iter().for_each(|opt| {
            if opt.is_none() && old_offset != self.col_offset {
                self.row_offset = old_offset;
            }
        });
        ret
    }

    pub(crate) fn move_to_beginning(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move to beginning request");
        self.row_offset = 0;
        self.move_y(rows, cols)
    }

    pub(crate) fn move_to_end(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move to end request");
        self.row_offset = std::u64::MAX - rows as u64;
        self.move_y(rows, cols)
    }
}
