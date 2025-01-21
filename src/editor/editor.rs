use crate::editor::{document, row, terminal};

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
}

impl Editor {
    pub fn new() -> Result<Self, std::io::Error> {
        let terminal = terminal::Terminal::new()?;
        let document = document::Document::open("test.txt");
        let display_height = terminal.height;
        let display_width = terminal.width;
        Ok(Editor {
            terminal,
            document,
            cursor_x: 0,
            cursor_y: 0,
            exit: false,
            display_height,
            display_width,
        })
    }

    pub fn run(&mut self) {
        self.terminal.clear_screen();
        loop {
            self.refresh_screen();
        }
    }

    pub fn draw_rows(&self) {
        for i in 0..self.display_height {
            self.terminal.clear_line();
            if i < self.document.rows() {
                let row = self.document.row(i).unwrap();
                let start = 0;
                let end = self.terminal.width;
                let rendered = row.render(start, end);
                println!("{}\r", rendered);
            }
        }
    }

    pub fn refresh_screen(&mut self) {
        self.terminal.hide_cursor();
        self.terminal.cursor_position(0, 0);

        if self.exit {
            self.terminal.clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.terminal.cursor_position(self.cursor_x, self.cursor_y);
        }
        self.terminal.show_cursor();
        self.terminal.flush().unwrap();
    }
}