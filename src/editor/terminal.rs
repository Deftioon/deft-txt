use std::io::{self, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};


pub struct Terminal {
    stdout: RawTerminal<std::io::Stdout>,
    pub height: usize,
    pub width: usize,
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        let (width, height) = termion::terminal_size()?;
        let stdout = stdout().into_raw_mode()?;

        Ok(Terminal {
            stdout,
            height: height as usize,
            width: width as usize,
        })
    }

    pub fn clear_screen(&self) {
        print!("{}", termion::clear::All);
    }

    pub fn clear_line(&self) {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }

    pub fn read_key(&self) -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn cursor_position(&self, x: usize, y: usize) {
        let x = x.saturating_add(1);
        let y = y.saturating_add(1);
        print!("{}", termion::cursor::Goto(x as u16, y as u16));
    }

    pub fn hide_cursor(&self) {
        print!("{}", termion::cursor::Hide);
    }

    pub fn show_cursor(&self) {
        print!("{}", termion::cursor::Show);
    }
}