use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::path::PathBuf;
use termion::screen::AlternateScreen;
use termion::terminal_size;

pub fn run(filename: Option<PathBuf>) -> std::io::Result<()> {
    let input = filename.unwrap_or_else(|| PathBuf::from("file.log"));

    let stdin = stdin();

    //TODO: ioctl invalid if run inside intellij's run.
    let (mut height, mut width) = terminal_size()?; // can be improved :)

    let mut file = File::open(input)?;

    let mut buffer = [0; 1000];

    {
        let mut screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
        write!(screen, "{}", termion::cursor::Goto(1, 1))?;
        screen.flush().unwrap();
        for c in stdin.keys() {
            file.read(&mut buffer).unwrap();
            write!(
                screen,
                "{}",
                String::from_utf8_lossy(buffer.as_ref()).replace("\n", "\n\r")
            )?;
            screen.flush().unwrap();
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Ctrl(c) => {
                    if c.to_string() == "c".to_string() {
                        break;
                    }
                }
                _ => continue,
            }
            screen.flush().unwrap();
        }
    }
    Ok(())
}
