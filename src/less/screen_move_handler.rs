use crate::less::reader::PagedReader;
use std::io::Result;

type ScreenToWrite = Option<String>;

pub struct ScreenMoveHandler {
    row_offset: u64,
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

    /// Doesn't trigger any movement, just rereads the current screen.
    pub(crate) fn reload(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
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

    pub(crate) fn initial_screen(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let (page, rows_red, cols_red) = self.paged_reader.read_file_paged(0, 0, rows, cols)?;
        self.row_offset += rows_red as u64;
        self.col_offset += cols_red as u64;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    // X axis:
    fn move_x(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_row_offset = std::cmp::max((self.row_offset as i64) - (rows as i64), 0) as u64;

        let (page, _rows_red, cols_red) =
            self.paged_reader
                .read_file_paged(fixed_row_offset, self.col_offset, rows, cols)?;
        self.col_offset += cols_red as u64;
        let ret = if cols_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    pub(crate) fn move_left(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        debug!("Received move left request");
        // I need to read not from the beginning of this page, but from the beginning of the last page. Thus * 2.
        let min_col_offset = (self.col_offset as i64) - (cols as i64) * 2;
        // we're not moving by rows:
        self.col_offset = std::cmp::max(min_col_offset, 0) as u64;
        self.move_x(rows, cols)
    }

    pub(crate) fn move_right(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        debug!("Received move right request");

        self.move_x(rows, cols)
    }

    // Y axis:

    fn move_y(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_col_offset = std::cmp::max((self.col_offset as i64) - (cols as i64), 0) as u64;
        let (page, rows_red, _cols_red) =
            self.paged_reader
                .read_file_paged(self.row_offset, fixed_col_offset, rows, cols)?;
        self.row_offset += rows_red as u64;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    pub(crate) fn move_up(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        debug!("Received move up request");

        // I need to read not from the beginning of this page, but from the beginning of the last page. Thus * 2.
        let min_row_offset = (self.row_offset as i64) - (rows as i64) * 2;
        self.row_offset = std::cmp::max(min_row_offset, 0) as u64;
        self.move_y(rows, cols)
    }

    pub(crate) fn move_down(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        debug!("Received move down request");
        self.move_y(rows, cols)
    }
}
