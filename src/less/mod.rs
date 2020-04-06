use crate::less::formats::Message;
use crate::less::reader::PagedReader;
use crate::less::screen_move_handler::ScreenMoveHandler;
use crossbeam_channel::Sender;
use memmap::{Mmap, MmapMut};
use signal_hook::{iterator::Signals, SIGINT, SIGWINCH};
use std::fs::File;
use std::io::{stdin, stdout, Stdout, Write};
use std::path::PathBuf;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;

mod formats;
mod reader;
mod screen_move_handler;

pub fn run(filename: PathBuf) -> std::io::Result<()> {
    //TODO: ioctl invalid if run inside intellij's run.
    let file_size = std::fs::metadata(&filename)?.len();
    let mmap = if file_size > 0 {
        let file = File::open(filename)?;
        unsafe { Mmap::map(&file).expect("failed to map the file") }
    } else {
        MmapMut::map_anon(1).expect("Anon mmap").make_read_only()?
    };
    let paged_reader = PagedReader::new(mmap);
    let screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
    let mut screen = termion::cursor::HideCursor::from(screen);
    let mut screen_move_handler: ScreenMoveHandler = ScreenMoveHandler::new(paged_reader);
    let (sender, receiver) = crossbeam_channel::bounded(100);
    spawn_stdin_handler(sender.clone());
    spawn_signal_handler(sender.clone());

    let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));

    let initial_screen = screen_move_handler.initial_screen(rows, cols)?;
    write_screen(&mut screen, initial_screen)?;

    'main_loop: for message in receiver {
        let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));
        let page = match message {
            Message::ScrollUpPage => screen_move_handler.move_up(rows, cols)?,
            Message::ScrollLeftPage => screen_move_handler.move_left(rows, cols)?,
            Message::ScrollRightPage => screen_move_handler.move_right(rows, cols)?,
            Message::ScrollDownPage => screen_move_handler.move_down(rows, cols)?,
            Message::Reload => screen_move_handler.reload(rows, cols)?,
            Message::Exit => break 'main_loop,
        };
        write_screen(&mut screen, page)?;
    }

    Ok(())
}

fn spawn_signal_handler(sender: Sender<Message>) {
    let signals = Signals::new(&[SIGWINCH, SIGINT]).expect("Signal handler");

    thread::spawn(move || {
        for sig in signals.forever() {
            let msg = match sig {
                signal_hook::SIGWINCH => Message::Reload,
                _ => Message::Exit,
            };
            sender.send(msg).unwrap();
            debug!("Received signal {:?}", sig);
        }
    });
}

fn spawn_stdin_handler(sender: Sender<Message>) {
    thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            let message = match c.unwrap() {
                Key::Char('q') => Message::Exit,
                Key::Ctrl(c) if c.to_string().as_str() == "c" => Message::Exit,
                Key::Left => Message::ScrollLeftPage,
                Key::Right => Message::ScrollRightPage,
                Key::Up => Message::ScrollUpPage,
                // Goes down by default.
                _ => Message::ScrollDownPage,
            };
            sender.send(message).unwrap();
        }
    });
}

/// If page is None, then we made a read which didn't return anything.
fn write_screen(
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
