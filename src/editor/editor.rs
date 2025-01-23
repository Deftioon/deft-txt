use crate::editor::{document, terminal};

pub static BLACK: &str = "\x1B[0;30m";
pub static RED: &str = "\x1B[0;31m";
pub static GREEN: &str = "\x1B[0;32m";
pub static YELLOW: &str = "\x1B[0;33m";
pub static BLUE: &str = "\x1B[0;34m";
pub static MAGENTA: &str = "\x1B[0;35m";
pub static CYAN: &str = "\x1B[0;36m";
pub static WHITE: &str = "\x1B[0;37m";
static ANSI_END: &str = "\x1B[0m";

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub struct Editor {
    terminal: terminal::Terminal,
    document: document::Document,
    cursor_x: usize,
    cursor_y: usize,
    exit: bool,
    display_height: usize,
    display_width: usize,
    display_x: usize,
    display_y: usize,
    file_cols: usize,
    file_rows: usize,
    previous_positions: Vec<(usize, usize)>,
}

impl Editor {
    pub fn new() -> Result<Self, std::io::Error> {
        let terminal = terminal::Terminal::new()?;
        let document = document::Document::open("test.txt");
        let doc_rows = document.rows();
        let doc_cols = document.cols();
        let display_height = terminal.height - 1;
        let display_width = terminal.width;
        Ok(Editor {
            terminal,
            document,
            cursor_x: 0,
            cursor_y: display_height/2,
            exit: false,
            display_height,
            display_width,
            display_x: 0,
            display_y: 0,
            file_cols: doc_cols,
            file_rows: doc_rows,
            previous_positions: Vec::new(),
        })
    }

    pub fn run(&mut self) {
        self.terminal.clear_screen();
        loop {
            if let Err(err) = self.refresh_screen() {
                self.kill(err);
            }
            if self.exit {
                break;
            }
            if let Err(err) = self.process_keys() {
                self.kill(err);
            }
        }
    }

    pub fn render_doc(&self) {
        for row_num in self.display_y..self.display_y + self.display_height {
            self.terminal.clear_line();
            if row_num < self.document.rows() {
                let row = self.document.row(row_num).unwrap();
                let start = self.display_x;
                let end = self.display_x + self.display_width;
                let rendered = row.render(start, end);
                println!("{}\r", rendered);
            }
            else {
                println!("~\r");
            }
        }
    }

    pub fn process_keys(&mut self) -> Result<(), std::io::Error> {
        let key = self.terminal.read_key()?;
        match key {
            termion::event::Key::Ctrl('q') => {
                self.exit = true;
            },
            termion::event::Key::Ctrl('s') => {
                self.save()?;
            },
            termion::event::Key::Up | termion::event::Key::Down | termion::event::Key::Left | termion::event::Key::Right => {
                self.move_cursor(key);
            },
            termion::event::Key::Backspace => {
                self.backspace();
            },
            termion::event::Key::Delete => {
                self.delete();
            },
            termion::event::Key::Char(_) => {
                self.insert_text(key);
            },
            _ => (),
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.document.save()
    }

    pub fn insert_text(&mut self, key: termion::event::Key) {
        match key {
            termion::event::Key::Char(c) => {
                let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
                row.insert_char(self.cursor_x, c);
                self.cursor_x = self.cursor_x.saturating_add(1);
            },
            _ => (),
        }
    }

    //TODO: Pressing backspace and insertion in rapid succession causes the next character to be deleted also.
    //TODO: does not handle backspace at the beginning of the line
    pub fn backspace(&mut self) {
        let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
        if self.cursor_x > 0 {
            row.remove_char(self.cursor_x - 1);
            self.cursor_x = self.cursor_x.saturating_sub(1);
        }
    }

    pub fn delete(&mut self) {
        let row = self.document.row_mut(self.display_y + self.cursor_y).unwrap();
        if self.cursor_x < row.str_len() {
            row.remove_char(self.cursor_x + 1);
        }
    }

    // TODO: Fix the cursor state save where it should return to previous position
    // Error occurs when moving up and down not corresponding to the previous position.
    pub fn move_cursor(&mut self, key: termion::event::Key) {
        let min_y = self.display_height/2;
        let min_x = 0;
        let max_y = self.file_rows - self.display_height/2;
        let max_x = self.file_cols;

        match key {
            termion::event::Key::Up => {
                    self.display_y = self.display_y.saturating_sub(1);
                    if self.display_y == 0 {
                        self.cursor_y = self.cursor_y.saturating_sub(1);
                    }
                    let curr_row_len = self.document.row(self.display_y+self.cursor_y).unwrap().str_len();
                    if self.cursor_x <= curr_row_len {
                        match self.previous_positions.pop() {
                            Some((x, _)) => {
                                self.cursor_x = x;
                            },
                            None => (),
                        }
                    }
                    if self.cursor_x > curr_row_len {
                        self.previous_positions.push((self.cursor_x, self.cursor_y + 1));
                        self.cursor_x = curr_row_len;
                    }
            },
            termion::event::Key::Down => {
                if self.display_y < max_y && self.cursor_y == min_y {
                    self.display_y = self.display_y.saturating_add(1);
                }
                if self.display_y == 0 {
                    self.cursor_y = self.cursor_y.saturating_add(1);
                }
                let curr_row_len = self.document.row(self.display_y+self.cursor_y).unwrap().str_len();
                if self.cursor_x <= curr_row_len {
                    match self.previous_positions.pop() {
                        Some((x, _)) => {
                            self.cursor_x = x;
                        },
                        None => (),
                    }
                }
                if self.cursor_x > curr_row_len {
                    self.previous_positions.push((self.cursor_x, self.cursor_y - 1));
                    self.cursor_x = curr_row_len;
                }
            },
            termion::event::Key::Left => {
                if self.cursor_x > min_x {
                    self.cursor_x = self.cursor_x.saturating_sub(1);
                }
                if self.cursor_x == min_x && self.display_x > 0 {
                    self.display_x = self.display_x.saturating_sub(1);
                }
                self.previous_positions = Vec::new();
            },
            termion::event::Key::Right => {
                if self.cursor_x == self.document.row(self.cursor_y).unwrap().str_len() {
                    return;
                }
                if self.cursor_x < self.display_width {
                    self.cursor_x = self.cursor_x.saturating_add(1);
                }
                if self.cursor_x == self.display_width && self.display_x < max_x {
                    self.display_x = self.display_x.saturating_add(1);
                }
                self.previous_positions = Vec::new();
            },
            _ => (),
        }
    }

    pub fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.hide_cursor();
        self.terminal.cursor_position(0, self.display_height/2);

        if self.exit {
            self.terminal.clear_screen();
            println!("Goodbye.\r");
        } else {
            self.render_doc();
            self.terminal.cursor_position(self.cursor_x, self.cursor_y);
        }
        self.terminal.show_cursor();
        self.terminal.flush()
    }

    fn kill(&self, err: std::io::Error) {
        self.terminal.clear_screen();
        panic!("Error: {}", err);
    }
}