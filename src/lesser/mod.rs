use crate::lesser::formats::Message;
use crate::lesser::reader::PagedReader;
use crate::lesser::screen_move_handler::ScreenMoveHandler;
use crossbeam_channel::Sender;
use io::{stdin, stdout, ErrorKind, Stdout, Write};
use log::debug;
use memmap::{Mmap, MmapMut};
use signal_hook::{iterator::Signals, SIGINT, SIGWINCH};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::{fs, io, thread};
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen;
use termion::{is_tty, terminal_size};

mod formats;
mod reader;
mod screen_move_handler;

pub fn run(filename: Option<PathBuf>) -> io::Result<()> {
    let screen = screen::AlternateScreen::from(stdout()).into_raw_mode()?;
    let mut screen = cursor::HideCursor::from(screen);

    let (sender, receiver) = crossbeam_channel::bounded(100);
    let mmap = if let Some(filename) = filename {
        let file_size = std::fs::metadata(&filename)?.len();
        if file_size > 0 {
            let file = File::open(filename)?;
            unsafe { Mmap::map(&file).expect("failed to map the file") }
        } else {
            MmapMut::map_anon(1).expect("Anon mmap").make_read_only()?
        }
    } else if !is_tty(&stdin()) {
        read_all_from_pipe()?
    } else {
        // exit Error, must specify an input!
        let error = io::Error::new(
            ErrorKind::InvalidInput,
            "Missing input. Use `lesser --help` for help",
        );
        return Err(error);
    };

    let paged_reader = PagedReader::new(mmap);
    let mut screen_move_handler: ScreenMoveHandler = ScreenMoveHandler::new(paged_reader);
    spawn_key_pressed_handler(sender.clone());
    spawn_signal_handler(sender)?;
    let (cols, rows) = terminal_size().unwrap_or((80, 80));

    let initial_screen = screen_move_handler.initial_screen(rows, cols)?;
    write_screen(&mut screen, initial_screen)?;

    for message in receiver {
        let (cols, rows) = terminal_size().unwrap_or((80, 80));
        let page = match message {
            Message::ScrollUpPage => screen_move_handler.move_up_page(rows, cols)?,
            Message::ScrollDownPage => screen_move_handler.move_down_page(rows, cols)?,
            Message::ScrollLeft => screen_move_handler.move_left(rows, cols)?,
            Message::ScrollRight => screen_move_handler.move_right(rows, cols)?,
            Message::ScrollUp => screen_move_handler.move_up(rows, cols)?,
            Message::ScrollDown => screen_move_handler.move_down(rows, cols)?,
            Message::ScrollToBeginning => screen_move_handler.move_to_top(rows, cols)?,
            Message::ScrollToEnd => screen_move_handler.move_to_end(rows, cols)?,
            Message::Reload => screen_move_handler.reload(rows, cols)?,
            Message::Exit => break,
        };

        write_screen(&mut screen, page)?;
    }
    Ok(())
}
fn signal_handler_thread_main(sender: Sender<Message>, signals: Signals) {
    for sig in signals.forever() {
        let msg = match sig {
            signal_hook::SIGWINCH => Message::Reload,
            _ => Message::Exit,
        };
        sender.send(msg).unwrap();
        debug!("Received signal {:?}", sig);
    }
}
fn spawn_signal_handler(sender: Sender<Message>) -> io::Result<JoinHandle<()>> {
    let signals = Signals::new(&[SIGWINCH, SIGINT])?;
    Ok(thread::spawn(move || {
        signal_handler_thread_main(sender, signals);
    }))
}

/// TODO: reading everything from the pipe is easy but not smart / efficient.
fn read_all_from_pipe() -> io::Result<Mmap> {
    //let (sender, receiver) = crossbeam_channel::unbounded();
    let tempdir = tempdir::TempDir::new("lesser")?;
    let path: PathBuf = tempdir.path().join("map_mut");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("Create file");
    let mut stdin = stdin();
    io::copy(&mut stdin, &mut file).expect("copy pipe input");
    Ok(unsafe { Mmap::map(&file).expect("mmap") })
}
fn key_pressed_handler_thread_main(sender: Sender<Message>) {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .expect("Open tty");

    // Can use the tty_input for keys while also reading stdin for data.
    let tty_input = tty
        .try_clone()
        .expect("Try clone")
        .into_raw_mode()
        .expect("Into raw mode");

    for c in tty_input.try_clone().unwrap().keys() {
        let message = match c.expect("read keys") {
            Key::Char('q') => Some(Message::Exit),
            Key::PageUp | Key::Char('b') => Some(Message::ScrollUpPage),
            Key::PageDown | Key::Char(' ') | Key::Char('f') => Some(Message::ScrollDownPage),
            Key::Left => Some(Message::ScrollLeft),
            Key::Down | Key::Char('\n') | Key::Char('e') | Key::Char('j') => {
                Some(Message::ScrollDown)
            }
            Key::Up | Key::Char('y') | Key::Char('k') => Some(Message::ScrollUp),
            Key::Right => Some(Message::ScrollRight),
            Key::Char('g') | Key::Home => Some(Message::ScrollToBeginning),
            Key::Char('G') | Key::End => Some(Message::ScrollToEnd),
            // Not-implemented keys do nothing
            _ => None,
        };
        if let Some(message) = message {
            sender.send(message).unwrap();
        }
    }
}

fn spawn_key_pressed_handler(sender: Sender<Message>) {
    thread::spawn(move || key_pressed_handler_thread_main(sender));
}

/// If page is None, then we made a read which didn't return anything.
fn write_screen(
    screen: &mut RawTerminal<screen::AlternateScreen<Stdout>>,
    page: Option<String>,
) -> io::Result<()> {
    match page {
        Some(page) => {
            write!(screen, "{}", termion::clear::All)?;
            write!(screen, "{}", termion::cursor::Goto(1, 1))?;
            write!(screen, "{}", page)?
        }
        None => write!(screen, "\x07")?,
    };
    screen.flush()
}
