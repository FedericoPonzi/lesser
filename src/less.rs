use crate::reader::PagedReader;
use memmap::{Mmap, MmapMut};
use std::fs::File;
use std::io::{stdin, stdout, Result, Stdout, Write};
use std::path::PathBuf;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;

/// If page is None, then we made a read which didn't return anything.
pub fn write_screen(
    screen: &mut RawTerminal<AlternateScreen<Stdout>>,
    page: Option<String>,
) -> std::io::Result<()> {
    if let Some(page) = page {
        write!(screen, "{}", termion::clear::All)?;
        write!(screen, "{}", termion::cursor::Goto(1, 1))?;
        write!(screen, "{}", page)?;
    }
    screen.flush().expect("Failed to flush");
    Ok(())
}

type ScreenToWrite = Option<String>;

pub fn run(filename: Option<PathBuf>) -> std::io::Result<()> {
    let input = filename.unwrap_or_else(|| PathBuf::from("file.log"));
    let stdin = stdin();
    //TODO: ioctl invalid if run inside intellij's run.
    let file_size = std::fs::metadata(&input)?.len();
    let mmap = if file_size > 0 {
        let file = File::open(input)?;
        unsafe { Mmap::map(&file).expect("failed to map the file") }
    } else {
        MmapMut::map_anon(1).expect("Anon mmap").make_read_only()?
    };
    let mut paged_reader = PagedReader::new(mmap);

    {
        let screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
        let mut screen = termion::cursor::HideCursor::from(screen);
        let mut screen_move_handler: ScreenMoveHandler = ScreenMoveHandler::new(paged_reader);
        let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));
        let initial_screen = screen_move_handler.initial_screen(rows, cols)?;
        write_screen(&mut screen, initial_screen)?;
        'in_loop: for c in stdin.keys() {
            let (cols, rows) = terminal_size()?;
            let res: ScreenToWrite = match c.unwrap() {
                Key::Char('q') => break 'in_loop,
                Key::Ctrl(c) if c.to_string().as_str() == "c" => {
                    break 'in_loop;
                }
                Key::Left => screen_move_handler.move_left(rows, cols)?,
                Key::Right => screen_move_handler.move_right(rows, cols)?,
                Key::Up => screen_move_handler.move_up(rows, cols)?,
                // Goes down by default.
                _ => screen_move_handler.move_down(rows, cols)?,
            };
            write_screen(&mut screen, res)?;
        }
    }
    Ok(())
}

struct ScreenMoveHandler {
    row_offset: u64,
    col_offset: u64,
    paged_reader: PagedReader,
}

impl ScreenMoveHandler {
    fn new(paged_reader: PagedReader) -> Self {
        ScreenMoveHandler {
            row_offset: 0,
            col_offset: 0,
            paged_reader,
        }
    }

    fn initial_screen(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
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
        let ret = if cols_red > 0 { Some(page) } else { None };
        Ok(ret)
    }
    fn move_left_new(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let min_col_offset = (self.col_offset as i64) - (cols as i64) * 2;
        // we're not moving by rows:
        self.col_offset = std::cmp::max(min_col_offset, 0) as u64;
        self.move_x(rows, cols)
    }
    fn move_right_new(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        self.move_x(rows, cols)
    }

    fn move_left(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_row_offset = std::cmp::max((self.row_offset as i64) - (rows as i64), 0) as u64;
        let min_col_offset = (self.col_offset as i64) - (cols as i64) * 2;
        // we're not moving by rows:
        self.col_offset = std::cmp::max(min_col_offset, 0) as u64;
        let (page, _rows_red, cols_red) =
            self.paged_reader
                .read_file_paged(fixed_row_offset, self.col_offset, rows, cols)?;
        let ret = if cols_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    fn move_right(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_row_offset = std::cmp::max((self.row_offset as i64) - (rows as i64), 0) as u64;
        let (page, _rows_red, cols_red) =
            self.paged_reader
                .read_file_paged(fixed_row_offset, self.col_offset, rows, cols)?;
        self.col_offset += cols_red as u64;
        let ret = if cols_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    // Y axis:
    fn move_up(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_col_offset = std::cmp::max((self.col_offset as i64) - (cols as i64), 0) as u64;

        let min_row_offset = (self.row_offset as i64) - (rows as i64) * 2;
        self.row_offset = std::cmp::max(min_row_offset, 0) as u64;
        let (page, rows_red, _cols_red) =
            self.paged_reader
                .read_file_paged(self.row_offset, fixed_col_offset, rows, cols)?;
        self.row_offset += rows_red as u64;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }

    fn move_down(&mut self, rows: u16, cols: u16) -> Result<ScreenToWrite> {
        let fixed_col_offset = std::cmp::max((self.col_offset as i64) - (cols as i64), 0) as u64;
        let (page, rows_red, _cols_red) =
            self.paged_reader
                .read_file_paged(self.row_offset, fixed_col_offset, rows, cols)?;
        let ret = if rows_red > 0 { Some(page) } else { None };
        Ok(ret)
    }
}
