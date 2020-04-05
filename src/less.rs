use crate::reader::PagedReader;
use memmap::Mmap;
use std::fs::File;
use std::io::{stdin, stdout, Stdout, Write};
use std::path::PathBuf;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;

pub fn write_screen(
    screen: &mut RawTerminal<AlternateScreen<Stdout>>,
    page: String,
    rows_red: usize,
) -> std::io::Result<()> {
    if rows_red > 0 {
        write!(screen, "{}", termion::clear::All)?;
        write!(screen, "{}", termion::cursor::Goto(1, 1))?;
        write!(screen, "{}", page)?;
    }
    Ok(())
}

pub fn run(filename: Option<PathBuf>) -> std::io::Result<()> {
    let input = filename.unwrap_or_else(|| PathBuf::from("file.log"));
    let stdin = stdin();
    //TODO: ioctl invalid if run inside intellij's run.
    let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));

    let file = File::open(input)?;

    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };
    let mut paged_reader = PagedReader::new(mmap);

    let mut row_offset: u64 = 0;
    let mut col_offset: u64 = 0;

    {
        let screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
        let mut screen = termion::cursor::HideCursor::from(screen);
        //initial screen:
        let (page, rows_red, cols_red) = paged_reader.read_file_paged(row_offset, 0, rows, cols)?;
        row_offset += rows_red as u64;
        col_offset += cols_red as u64;
        write_screen(&mut screen, page, rows_red)?;

        screen.flush().unwrap();
        for c in stdin.keys() {
            let (cols, rows) = terminal_size()?;
            let fixed_col_offset = std::cmp::max((col_offset as i64) - (cols as i64), 0) as u64;
            let fixed_row_offset = std::cmp::max((row_offset as i64) - (rows as i64), 0) as u64;

            match c.unwrap() {
                Key::Char('q') => break,
                Key::Ctrl(c) => {
                    if c.to_string().as_str() == "c" {
                        break;
                    }
                }
                Key::Up => {
                    let min_row_offset = (row_offset as i64) - (rows as i64) * 2;
                    row_offset = std::cmp::max(min_row_offset, 0) as u64;
                    let (page, rows_red, _cols_red) =
                        paged_reader.read_file_paged(row_offset, fixed_col_offset, rows, cols)?;
                    row_offset += rows_red as u64;
                    write_screen(&mut screen, page, rows_red)?;
                }
                Key::Left => {
                    let min_col_offset = (col_offset as i64) - (cols as i64) * 2;
                    // we're not moving by rows:
                    col_offset = std::cmp::max(min_col_offset, 0) as u64;
                    let (page, _rows_red, cols_red) =
                        paged_reader.read_file_paged(fixed_row_offset, col_offset, rows, cols)?;
                    col_offset += cols_red as u64;
                    write_screen(&mut screen, page, cols_red)?;
                }
                Key::Right => {
                    let (page, _rows_red, cols_red) =
                        paged_reader.read_file_paged(fixed_row_offset, col_offset, rows, cols)?;
                    col_offset += cols_red as u64;
                    write_screen(&mut screen, page, cols_red)?;
                }
                _ => {
                    let (page, rows_red, _cols_red) =
                        paged_reader.read_file_paged(row_offset, fixed_col_offset, rows, cols)?;
                    row_offset += rows_red as u64;
                    write_screen(&mut screen, page, rows_red)?;
                }
            }
            screen.flush().unwrap();
        }
    }
    Ok(())
}
