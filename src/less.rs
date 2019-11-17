use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::reader::{find_new_lines, read_file_paged};
use memmap::Mmap;
use std::path::PathBuf;
use termion::screen::AlternateScreen;
use termion::terminal_size;

pub fn run(filename: Option<PathBuf>) -> std::io::Result<()> {
    let input = filename.unwrap_or_else(|| PathBuf::from("file.log"));
    let stdin = stdin();
    //TODO: ioctl invalid if run inside intellij's run.
    let (mut cols, mut rows) = terminal_size()?; // can be improved :)

    let mut file = File::open(input)?;
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let mut row_offset: u64 = 0;
    let mut column_offset: u64 = 0;

    {
        let mut screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
        let res = read_file_paged(&mmap, row_offset, 0, rows, cols);
        row_offset += rows as u64;
        write!(screen, "{}", termion::cursor::Goto(1, 1))?;
        write!(screen, "{}", res.unwrap())?;
        screen.flush().unwrap();

        for c in stdin.keys() {
            let (mut cols, mut rows) = terminal_size()?; // can be improved :)
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Ctrl(c) => {
                    if c.to_string() == "c".to_string() {
                        break;
                    }
                }
                _ => {
                    let res = read_file_paged(&mmap, row_offset, 0, rows, cols);
                    row_offset += rows as u64;
                    write!(screen, "{}", termion::cursor::Goto(1, 1))?;
                    write!(screen, "{}", res.unwrap())?;
                }
            }
            screen.flush().unwrap();
        }
    }
    Ok(())
}
