use crate::less::formats::Message;
use crate::less::reader::PagedReader;
use crate::less::screen_move_handler::ScreenMoveHandler;
use crossbeam_channel::Sender;
use memmap::{Mmap, MmapMut};
use signal_hook::{iterator::Signals, SIGINT, SIGWINCH};
use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, ErrorKind, Stdout, Write};
use std::path::PathBuf;
use std::{fs, thread};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{is_tty, terminal_size};

mod formats;
mod reader;
mod screen_move_handler;

pub fn run(filename: Option<PathBuf>) -> std::io::Result<()> {
    let screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
    let mut screen = termion::cursor::HideCursor::from(screen);

    let (sender, receiver) = crossbeam_channel::bounded(100);
    //TODO: ioctl invalid if run inside intellij's run.
    let mmap = if let Some(filename) = filename {
        let file_size = std::fs::metadata(&filename)?.len();
        if file_size > 0 {
            let file = File::open(filename)?;
            unsafe { Mmap::map(&file).expect("failed to map the file") }
        } else {
            MmapMut::map_anon(1).expect("Anon mmap").make_read_only()?
        }
    } else {
        if !is_tty(&stdin()) {
            read_all_from_pipe()
        } else {
            // Error, must specify an input!
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Missing filename (\"lesser --help\" for help)",
            ));
        }
    };

    let paged_reader = PagedReader::new(mmap);
    let mut screen_move_handler: ScreenMoveHandler = ScreenMoveHandler::new(paged_reader);
    spawn_key_pressed_handler(sender.clone());
    spawn_signal_handler(sender.clone());
    let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));

    let initial_screen = screen_move_handler.initial_screen(rows, cols)?;
    write_screen(&mut screen, initial_screen)?;

    for message in receiver {
        let (cols, rows) = terminal_size().unwrap_or_else(|_| (80, 80));
        let page = match message {
            Message::ScrollUpPage => screen_move_handler.move_up_page(rows, cols)?,
            Message::ScrollLeftPage => screen_move_handler.move_left_page(rows, cols)?,
            Message::ScrollRightPage => screen_move_handler.move_right_page(rows, cols)?,
            Message::ScrollDownPage => screen_move_handler.move_down_page(rows, cols)?,
            Message::ScrollLeft => screen_move_handler.move_left(rows, cols)?,
            Message::ScrollRight => screen_move_handler.move_right(rows, cols)?,
            Message::Reload => screen_move_handler.reload(rows, cols)?,
            Message::Exit => break,
            _ => {
                debug!("Message unimplemented: {:?}", message);
                screen_move_handler.move_down_page(rows, cols)?
            }
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

/// TODO: reading everything from the pipe is easy but not smart / efficient.
fn read_all_from_pipe() -> Mmap {
    //let (sender, receiver) = crossbeam_channel::unbounded();
    let tempdir = tempdir::TempDir::new("lesser").expect("Tempdir");
    let path: PathBuf = tempdir.path().join("map_mut");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("Create file");
    let mut stdin = stdin();
    std::io::copy(&mut stdin, &mut file).expect("copy pipe input");
    unsafe { Mmap::map(&file).expect("mmap") }
}

fn spawn_key_pressed_handler(sender: Sender<Message>) {
    thread::spawn(move || {
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
                Key::Char('q') => Message::Exit,
                Key::Ctrl(c) if c.to_string().as_str() == "c" => Message::Exit,
                Key::Left => Message::ScrollLeftPage,
                Key::Right => Message::ScrollRightPage,
                Key::Up => Message::ScrollUpPage,
                Key::PageUp => Message::ScrollUpPage,
                Key::PageDown => Message::ScrollDownPage,
                Key::Char('h') => Message::ScrollLeft,
                Key::Char('j') => Message::ScrollDownPage,
                Key::Char('k') => Message::ScrollUpPage,
                Key::Char('l') => Message::ScrollRight,

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
        screen.flush().expect("Failed to flush");
    }
    Ok(())
}
