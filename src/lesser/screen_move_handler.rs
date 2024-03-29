use crate::lesser::reader::PagedReader;
use log::debug;
use std::cmp;
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
        let (page, rows_read, cols_read) = self.paged_reader.read_file_paged(0, 0, rows, cols)?;
        self.row_offset += rows_read as u64;
        self.col_offset += cols_read as u64;
        let ret = if rows_read > 0 { Some(page) } else { None };
        Ok(ret)
    }

    /// Doesn't trigger any movement, just rereads the current screen.
    pub(crate) fn reload(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        // reset the index back to the start of the line:
        self.col_offset = 0;
        // Re read this page:
        self.row_offset = self.row_offset.saturating_sub(rows as u64);

        let (page, rows_read, cols_read) =
            self.paged_reader
                .read_file_paged(self.row_offset, self.col_offset, rows, cols)?;
        self.row_offset += rows_read as u64;
        self.col_offset += cols_read as u64;

        Ok(if rows_read > 0 { Some(page) } else { None })
    }

    // X axis: read the page and moves the col offset position
    // from self.col_offset to self.col_offset+ cols_read.
    fn move_x(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        // Re read the same rows
        let fixed_row_offset = self.row_offset.saturating_sub(rows as u64);

        let (page, _rows_read, cols_read) =
            self.paged_reader
                .read_file_paged(fixed_row_offset, self.col_offset, rows, cols)?;
        self.col_offset += cols_read as u64;
        let ret = if cols_read > 0 { Some(page) } else { None };
        Ok(ret)
    }

    /// Move left one column
    pub(crate) fn move_left(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        const MOVEMENT: u64 = 10;
        debug!("Received move right request");

        // we're not moving by rows:
        self.col_offset = self.col_offset.saturating_sub(cols as u64 + MOVEMENT);
        self.move_x(rows, cols)
    }

    pub(crate) fn move_right(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        const MOVEMENT: u64 = 10;
        debug!("Received move right request by {}", MOVEMENT);
        // This is used to avoid going back one screen if the move_x has returned None
        // (e.g it hasn't read anything).
        let old_offset = self.col_offset;
        self.col_offset = self.col_offset.saturating_sub(cols as u64 - MOVEMENT);
        let ret = self.move_x(rows, cols)?;
        if ret.is_none() && old_offset != self.col_offset {
            self.col_offset = old_offset;
        }
        Ok(ret)
    }

    // Y axis:

    fn move_y(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        let fixed_col_offset = self.col_offset.saturating_sub(cols as u64);
        let (page, rows_read, _cols_read) =
            self.paged_reader
                .read_file_paged(self.row_offset, fixed_col_offset, rows, cols)?;
        self.row_offset = cmp::min(self.row_offset, self.paged_reader.cached_rows() as u64);
        self.row_offset += rows_read as u64;

        Ok(if rows_read > 0 { Some(page) } else { None })
    }

    pub(crate) fn move_down_page(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move down page request");
        self.move_y(rows, cols)
    }
    pub(crate) fn move_up_page(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move up request");
        // I need to read not from the beginning of this page, but from the beginning of the last page. Thus * 2.
        self.row_offset = self.row_offset.saturating_sub(rows as u64 * 2);
        self.move_y(rows, cols)
    }

    pub(crate) fn move_up(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move up request");
        self.row_offset = self.row_offset.saturating_sub(rows as u64 + 1);
        self.move_y(rows, cols)
    }

    pub(crate) fn move_down(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        const MOVEMENT: u64 = 1;
        debug!("Received move up request");
        // This is used to avoid going back one screen if the move_x has returned None
        // (e.g it hasn't read anything).
        let old_offset = self.row_offset;
        self.row_offset = self.row_offset.saturating_sub(rows as u64 - MOVEMENT);
        let ret = self.move_y(rows, cols)?;
        if ret.is_none() && old_offset != self.row_offset {
            self.row_offset = old_offset;
        }
        Ok(ret)
    }

    pub(crate) fn move_to_top(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move to beginning request");
        self.row_offset = 0;
        self.move_y(rows, cols)
    }

    pub(crate) fn move_to_end(&mut self, rows: u16, cols: u16) -> Result<PageToPrint> {
        debug!("Received move to end request");
        self.row_offset = u64::MAX - rows as u64;
        self.move_y(rows, cols)
    }
}
